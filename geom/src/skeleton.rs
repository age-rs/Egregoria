#![allow(clippy::ptr_arg)]

use crate::{intersection_point, Ray, Segment, Vec2, Vec3};
use ordered_float::OrderedFloat;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};

const EPSILON: f32 = 0.00001;

fn window<T>(lst: &[T]) -> impl Iterator<Item = (&T, &T, &T)> {
    let prevs = lst.iter().cycle().skip(lst.len() - 1);
    let items = lst.iter();
    let nexts = lst.iter().cycle().skip(1);
    prevs.zip(items).zip(nexts).map(|((a, b), c)| (a, b, c))
}

#[allow(clippy::float_cmp)]
fn approx_equal(a: f32, b: f32) -> bool {
    (a == b) || ((a - b).abs() <= f32::max(a.abs(), b.abs()) * 0.001)
}

fn approx_equal_vec(a: Vec2, b: Vec2) -> bool {
    approx_equal(a.x, b.x) && approx_equal(a.y, b.y)
}

fn normalize_contour(contour: &[Vec2]) -> Vec<Vec2> {
    window(&contour.iter().copied().rev().collect::<Vec<_>>())
        .filter(|&(&prev, &point, &next)| {
            !approx_equal_vec(point, next)
                && !approx_equal_vec((point - prev).normalize(), (next - point).normalize())
        })
        .map(|(_, p, _)| *p)
        .collect()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VertexID(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LavID(pub usize);

type Vertices = Vec<LAVertex>;
type Lavs = Vec<LAV>;

#[derive(Debug)]
struct SplitEvent {
    distance: f32,
    intersection_point: Vec2,
    vertex: VertexID,
    opposite_edge: Segment,
}

impl SplitEvent {
    pub fn new(
        distance: f32,
        intersection_point: Vec2,
        vertex: VertexID,
        opposite_edge: Segment,
    ) -> Self {
        SplitEvent {
            distance,
            intersection_point,
            vertex,
            opposite_edge,
        }
    }
}

#[derive(Debug)]
struct EdgeEvent {
    distance: f32,
    intersection_point: Vec2,
    vertex_a: VertexID,
    vertex_b: VertexID,
}

impl EdgeEvent {
    pub fn new(
        distance: f32,
        intersection_point: Vec2,
        vertex_a: VertexID,
        vertex_b: VertexID,
    ) -> Self {
        EdgeEvent {
            distance,
            intersection_point,
            vertex_a,
            vertex_b,
        }
    }
}

#[derive(Debug)]
enum Event {
    Split(SplitEvent),
    Edge(EdgeEvent),
}

impl Event {
    fn distance(&self) -> f32 {
        match self {
            Event::Split(s) => s.distance,
            Event::Edge(e) => e.distance,
        }
    }

    fn intersection_point(&self) -> Vec2 {
        match self {
            Event::Split(s) => s.intersection_point,
            Event::Edge(e) => e.intersection_point,
        }
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.distance() == other.distance()
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance().partial_cmp(&other.distance())
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

#[derive(Debug)]
struct OriginalEdge {
    edge: Segment,
    bisector_left: Ray,
    bisector_right: Ray,
}

impl OriginalEdge {
    pub fn new(edge: Segment, bisector_left: Ray, bisector_right: Ray) -> Self {
        OriginalEdge {
            edge,
            bisector_left,
            bisector_right,
        }
    }
}

#[derive(Debug)]
pub struct Subtree {
    pub source: Vec2,
    pub height: f32,
    pub sinks: Vec<Vec2>,
}

impl Subtree {
    pub fn new(source: Vec2, height: f32, sinks: Vec<Vec2>) -> Self {
        Subtree {
            source,
            height,
            sinks,
        }
    }
}

#[derive(Clone)]
struct LAVertex {
    pub(crate) id: VertexID,
    pub(crate) point: Vec2,
    pub(crate) edge_left: Segment,
    pub(crate) edge_right: Segment,
    pub(crate) prev: Option<VertexID>,
    pub(crate) next: Option<VertexID>,
    pub(crate) lav: Option<LavID>,
    pub(crate) valid: bool,
    pub(crate) is_reflex: bool,
    pub(crate) bisector: Ray,
}

impl LAVertex {
    #[allow(clippy::let_and_return)]
    pub(crate) fn next_event(&self, slav: &SLAV, vs: &Vertices) -> Option<Event> {
        let mut events = vec![];
        if self.is_reflex {
            // a reflex vertex may generate a split event
            // split events happen when a vertex hits an opposite edge, splitting the polygon in two.
            // println!("looking for split candidates for vertex {:?}", self.point);

            for edge in &slav.original_edges {
                if edge.edge == self.edge_left || edge.edge == self.edge_right {
                    continue;
                }
                // println!("\t considering EDGE {:?}", edge.edge);

                // a potential b is at the intersection of between our own bisector and the bisector of the
                // angle between the tested edge and any one of our own edges.
                // we choose the "less parallel" edge (in order to exclude a potentially parallel edge)

                let left_dot = self
                    .edge_left
                    .vec()
                    .normalize()
                    .dot(edge.edge.vec().normalize())
                    .abs();
                let right_dot = self
                    .edge_right
                    .vec()
                    .normalize()
                    .dot(edge.edge.vec().normalize())
                    .abs();
                let self_edge = if left_dot < right_dot {
                    self.edge_left
                } else {
                    self.edge_right
                };

                // // println!("\t\t trying {:?} against {:?}", self_edge, &edge.edge);

                if let Some(i) = self_edge.intersection_point(&edge.edge, true) {
                    if approx_equal_vec(i, self.point) {
                        continue;
                    }
                    // println!("\t\t found intersection {:?}", i);

                    let lin_vec = (self.point - i).normalize();
                    let mut ed_vec = edge.edge.vec().normalize();
                    if lin_vec.dot(ed_vec) < 0.0 {
                        ed_vec = -ed_vec;
                    }

                    let bisect_vec = ed_vec + lin_vec;
                    if approx_equal_vec(bisect_vec, Vec2::ZERO) {
                        continue;
                    }
                    let bisector = Ray::new(i, bisect_vec);
                    let b = intersection_point(bisector, self.bisector);
                    if b.is_none() {
                        continue;
                    }
                    let b = b.unwrap();

                    let left = edge
                        .bisector_left
                        .dir
                        .normalize()
                        .cross((b - edge.bisector_left.from).normalize())
                        > -EPSILON;
                    let right = edge
                        .bisector_right
                        .dir
                        .normalize()
                        .cross((b - edge.bisector_right.from).normalize())
                        < EPSILON;
                    let b_edge = edge
                        .edge
                        .vec()
                        .normalize()
                        .cross((b - edge.edge.src).normalize())
                        < EPSILON;

                    if !(left && right && b_edge) {
                        // println!(
                        //    "\t\tDiscarded candidate {:?} ({}-{}-{})",
                        //    b, x_left, x_right, x_edge
                        //);
                        continue;
                    }

                    // println!("\t\tFound valid candidate {:?}", b);
                    events.push(Event::Split(SplitEvent::new(
                        edge.edge.project(b).distance(b),
                        b,
                        self.id,
                        edge.edge,
                    )));
                }
            }
        }

        let i_prev = intersection_point(self.bisector, vs[self.prev.unwrap().0].bisector);
        let i_next = intersection_point(self.bisector, vs[self.next.unwrap().0].bisector);

        if let Some(i_prev) = i_prev {
            events.push(Event::Edge(EdgeEvent::new(
                self.edge_left.project(i_prev).distance(i_prev),
                i_prev,
                self.prev.unwrap(),
                self.id,
            )));
        }

        if let Some(i_next) = i_next {
            events.push(Event::Edge(EdgeEvent::new(
                self.edge_right.project(i_next).distance(i_next),
                i_next,
                self.id,
                self.next.unwrap(),
            )));
        }

        let mut min_ev = None;
        let mut min_v: f32 = f32::INFINITY;
        for ev in events {
            let d = self.point.distance2(ev.intersection_point());
            if d < min_v {
                min_v = d;
                min_ev = Some(ev);
            }
        }
        //if let Some(ref e) = min_ev {
        //    println!("Generated new event for {:?}: {:?}", self.id, e)
        //}
        min_ev
    }

    pub fn invalidate(&mut self, lavs: &mut Lavs) {
        self.valid = false;
        if let Some(id) = self.lav {
            let lav = &mut lavs[id.0];
            if lav.head.unwrap() == self.id {
                lav.head = self.next;
            }
            self.lav = None;
        }
    }

    pub fn invalidate_known(&mut self, lav: &mut LAV) {
        self.valid = false;

        if let Some(id) = self.lav {
            if id != lav.id {
                panic!("invalidating known lav but it's not this vertices lav!");
            }
            if lav.head.unwrap() == self.id {
                lav.head = self.next;
            }
            self.lav = None;
        }
    }
}

impl LAVertex {
    pub fn new(
        id: VertexID,
        point: Vec2,
        edge_left: Segment,
        edge_right: Segment,
        direction_vectors: Option<(Vec2, Vec2)>,
    ) -> Self {
        let creator_vectors = (-edge_left.vec().normalize(), edge_right.vec().normalize());
        let direction_vectors = if let Some(v) = direction_vectors {
            v
        } else {
            creator_vectors
        };

        let is_reflex = direction_vectors.0.cross(direction_vectors.1) < 0.0;
        let bisector = Ray {
            from: point,
            dir: (creator_vectors.0 + creator_vectors.1) * if is_reflex { -1.0 } else { 1.0 },
        };
        // println!(
        //    "created vertex {:?} {:?} {:?} {:?} {:?} {:?}",
        //    id, point, edge_left, edge_right, is_reflex, bisector
        //);

        LAVertex {
            id,
            point,
            edge_left,
            edge_right,
            prev: None,
            next: None,
            lav: None,
            valid: true,
            is_reflex,
            bisector,
        }
    }
}

struct SLAV {
    lavs: Vec<LavID>,
    original_edges: Vec<OriginalEdge>,
}

impl SLAV {
    pub fn handle_edge_event(
        &mut self,
        vs: &mut Vertices,
        lavs: &mut Lavs,
        event: EdgeEvent,
    ) -> (Option<Subtree>, Vec<Event>) {
        let mut sinks = vec![];
        let mut events = vec![];

        let va = &vs[event.vertex_a.0];
        let vb = &vs[event.vertex_b.0];
        let vap = va.point;
        let vbp = vb.point;

        let lav = &mut lavs[va.lav.unwrap().0];

        if va.prev == vb.next {
            // println!(
            //    "{} Peak event at intersection {:?} from <{:?},{:?},{:?}> in {:?}",
            //    event.distance,
            //    event.intersection_point,
            //    event.vertex_a,
            //    event.vertex_b,
            //    va.prev,
            //    lav.id
            //);
            self.lavs
                .remove(self.lavs.iter().position(|&x| x == lav.id).unwrap());
            for vertex in lav.iter_keys(vs) {
                let vertex = &mut vs[vertex.0];
                sinks.push(vertex.point);
                vertex.invalidate_known(lav);
            }
        } else {
            // println!(
            //    "{} Edge event at intersection {:?} from <{:?},{:?}> in {:?}",
            //    event.distance, event.intersection_point, event.vertex_a, event.vertex_b, lav.id
            //);

            let new_vertex =
                lav.unify(vs, event.vertex_a, event.vertex_b, event.intersection_point);
            let h = lav.head.unwrap();
            if h == event.vertex_a || h == event.vertex_b {
                lav.head = Some(new_vertex);
            }
            sinks.push(vap);
            sinks.push(vbp);
            if let Some(next_event) = vs[new_vertex.0].next_event(self, vs) {
                events.push(next_event);
            }
        }

        (
            Some(Subtree::new(
                event.intersection_point,
                event.distance,
                sinks,
            )),
            events,
        )
    }

    pub fn handle_split_event(
        &mut self,
        vs: &mut Vertices,
        lavs: &mut Lavs,
        event: SplitEvent,
    ) -> (Option<Subtree>, Vec<Event>) {
        let v = vs[event.vertex.0].clone();
        // println!(
        //    "{} Split event at intersection {:?} from vertex {:?}, for edge {:?} in {:?}",
        //    event.distance, event.intersection_point, event.vertex, event.opposite_edge, v.lav,
        //);

        let mut sinks = vec![v.point];
        let mut vertices: Vec<VertexID> = vec![];

        let mut x = None;
        let mut y = None;

        let norm = event.opposite_edge.vec().normalize();

        for &l in &self.lavs {
            for v in lavs[l.0].iter_keys(vs) {
                let v = &vs[v.0];
                // println!("{:?} in {:?}", v.id, v.lav);

                if approx_equal_vec(norm, v.edge_left.vec().normalize())
                    && approx_equal_vec(event.opposite_edge.src, v.edge_left.src)
                {
                    x = Some(v.id);
                    y = v.prev;
                } else if approx_equal_vec(norm, v.edge_right.vec().normalize())
                    && approx_equal_vec(event.opposite_edge.src, v.edge_right.src)
                {
                    y = Some(v.id);
                    x = v.next;
                }

                if let Some(x_id) = x {
                    let xx = &vs[x_id.0];
                    let yy = &vs[y.unwrap().0];
                    let left = yy
                        .bisector
                        .dir
                        .normalize()
                        .cross((event.intersection_point - yy.point).normalize())
                        >= -EPSILON;
                    let right = xx
                        .bisector
                        .dir
                        .normalize()
                        .cross((event.intersection_point - xx.point).normalize())
                        <= EPSILON;

                    // println!(
                    //    "Vertex {:?} holds edge as {} edge ({}, {})",
                    //    v.id,
                    //    if x_id == v.id { "left" } else { "right" },
                    //    left,
                    //    right
                    //);

                    if left && right {
                        break;
                    } else {
                        x = None;
                        y = None;
                    }
                }
            }
        }

        if x.is_none() {
            // println!(
            //    "Failed split event {:?} (equivalent edge event is expected to follow)",
            //    event
            //);
            return (None, vec![]);
        }

        let x = x.unwrap();
        let y = y.unwrap();

        let v1 = VertexID(vs.len());
        vs.push(LAVertex::new(
            v1,
            event.intersection_point,
            v.edge_left,
            event.opposite_edge,
            None,
        ));

        vs[v1.0].prev = Some(v.prev.unwrap());
        vs[v1.0].next = Some(x);
        vs[v.prev.unwrap().0].next = Some(v1);
        vs[x.0].prev = Some(v1);

        let v2 = VertexID(vs.len());
        vs.push(LAVertex::new(
            v2,
            event.intersection_point,
            event.opposite_edge,
            v.edge_right,
            None,
        ));

        vs[v2.0].prev = Some(y);
        vs[v2.0].next = Some(v.next.unwrap());
        vs[v.next.unwrap().0].prev = Some(v2);
        vs[y.0].next = Some(v2);

        let new_lavs;
        self.remove_lav(v.lav.unwrap());

        // println!("v1:{:?} v2:{:?}", vs[v1].point, vs[v2].point);

        if v.lav.unwrap() != vs[x.0].lav.unwrap() {
            self.remove_lav(vs[x.0].lav.unwrap());
            new_lavs = vec![LAV::from_chain(lavs, vs, Some(v1))];
        } else {
            new_lavs = vec![
                LAV::from_chain(lavs, vs, Some(v1)),
                LAV::from_chain(lavs, vs, Some(v2)),
            ];
        }

        for l in new_lavs {
            if lavs[l.0].len > 2 {
                self.lavs.push(l);
                vertices.push(lavs[l.0].head.unwrap());
            } else {
                // println!(
                //    "LAV {:?} has collapsed into the line {:?}--{:?}",
                //    l,
                //    vs[lavs[l].head.unwrap()].point,
                //    vs[vs[lavs[l].head.unwrap()].next.unwrap()].point
                //);

                sinks.push(vs[vs[lavs[l.0].head.unwrap().0].next.unwrap().0].point);
                for v in lavs[l.0].iter_keys(vs) {
                    vs[v.0].invalidate_known(&mut lavs[l.0])
                }
            }
        }

        let mut events = vec![];
        for vertex in vertices {
            events.extend(vs[vertex.0].next_event(self, vs).into_iter());
        }

        vs[event.vertex.0].invalidate(lavs);

        (
            Some(Subtree::new(
                event.intersection_point,
                event.distance,
                sinks,
            )),
            events,
        )
    }

    fn remove_lav(&mut self, id: LavID) {
        self.lavs
            .remove(self.lavs.iter().position(|x| *x == id).unwrap());
    }
}

impl SLAV {
    pub fn new(vs: &mut Vertices, lavs: &mut Lavs, polygon: &[Vec2], holes: &[&[Vec2]]) -> Self {
        let lavs_l = std::iter::once(normalize_contour(polygon))
            .chain(holes.iter().map(|x| normalize_contour(x)))
            .map(|x| LAV::from_polygon(lavs, vs, &x))
            .collect::<Vec<_>>();

        let original_edges: Vec<OriginalEdge> = lavs
            .iter()
            .map(|x| x.iter_keys(vs))
            .flatten()
            .map(|vertex| {
                let vertex = &vs[vertex.0];
                let prev = &vs[vertex.prev.unwrap().0];
                OriginalEdge::new(
                    Segment::new(prev.point, vertex.point),
                    prev.bisector,
                    vertex.bisector,
                )
            })
            .collect();

        SLAV {
            lavs: lavs_l,
            original_edges,
        }
    }
}

struct LAV {
    id: LavID,
    head: Option<VertexID>,
    len: usize,
}

impl LAV {
    pub fn iter_keys(&self, vs: &Vertices) -> Vec<VertexID> {
        if let Some(head) = self.head {
            std::iter::successors(Some(head), move |&cur| {
                vs[cur.0].next.filter(|&x| x != head)
            })
            .collect()
        } else {
            vec![]
        }
    }

    pub fn from_polygon(lavs: &mut Lavs, vs: &mut Vertices, polygon: &[Vec2]) -> LavID {
        let lav_id = LavID(lavs.len());
        let mut len = 0;
        let mut head = None;
        for (&prev, &point, &next) in window(polygon) {
            len += 1;
            let vertex = VertexID(vs.len());
            vs.push(LAVertex::new(
                vertex,
                point,
                Segment::new(prev, point),
                Segment::new(point, next),
                None,
            ));
            vs[vertex.0].lav = Some(lav_id);
            if let Some(head) = head {
                vs[vertex.0].next = Some(head);
                let prev_head = vs[head.0].prev;
                vs[vertex.0].prev = prev_head;
                vs[prev_head.unwrap().0].next = Some(vertex);
                vs[head.0].prev = Some(vertex);
            } else {
                head = Some(vertex);
                vs[vertex.0].prev = Some(vertex);
                vs[vertex.0].next = Some(vertex);
            }
        }
        lavs.push(LAV {
            id: lav_id,
            head,
            len,
        });
        lav_id
    }

    pub fn from_chain(lavs: &mut Lavs, vs: &mut Vertices, head: Option<VertexID>) -> LavID {
        let lav_id = LavID(lavs.len());
        lavs.push({
            let mut l = LAV {
                id: lav_id,
                head,
                len: 0,
            };
            for vertex in l.iter_keys(vs) {
                l.len += 1;
                vs[vertex.0].lav = Some(lav_id);
            }
            l
        });
        lav_id
    }

    pub fn unify(
        &mut self,
        vs: &mut Vertices,
        vertex_a: VertexID,
        vertex_b: VertexID,
        point: Vec2,
    ) -> VertexID {
        let va = &vs[vertex_a.0].clone();
        let vb = &vs[vertex_b.0].clone();

        let replacement = VertexID(vs.len());
        vs.push(LAVertex::new(
            replacement,
            point,
            va.edge_left,
            vb.edge_right,
            Some((vb.bisector.dir.normalize(), va.bisector.dir.normalize())),
        ));

        vs[replacement.0].lav = Some(self.id);

        let h = self.head.unwrap();
        if h == vertex_a || h == vertex_b {
            self.head = Some(replacement);
        }

        vs[va.prev.unwrap().0].next = Some(replacement);
        vs[vb.next.unwrap().0].prev = Some(replacement);

        vs[replacement.0].prev = va.prev;
        vs[replacement.0].next = vb.next;

        vs[vertex_a.0].invalidate_known(self);
        vs[vertex_b.0].invalidate_known(self);

        self.len -= 1;

        replacement
    }
}

/// In highly symmetrical shapes with reflex vertices multiple sources may share the same
/// location. This function merges those sources.
fn merge_sources(skeleton: &mut Vec<Subtree>) {
    let mut sources: HashMap<Vec2, usize> = HashMap::new();
    let mut to_remove = vec![];
    let mut to_add = vec![];
    for (i, p) in skeleton.iter().enumerate() {
        if let Some(&source_index) = sources.get(&p.source) {
            for &sink in &p.sinks {
                if !skeleton[source_index].sinks.contains(&sink) {
                    to_add.push((source_index, sink));
                }
            }
            to_remove.push(i);
        } else {
            sources.insert(p.source, i);
        }
    }
    for (i, sink) in to_add {
        skeleton[i].sinks.push(sink);
    }
    for i in to_remove.into_iter().rev() {
        skeleton.swap_remove(i);
    }
}

/// Compute the straight skeleton of a polygon.
///
/// The polygon should be given as a list of vertices in counter-clockwise order.
///
/// Please note that the y-axis goes upwards, so specify your ordering accordingly.
///
/// Returns the straight skeleton as a list of "subtrees", which are in the form of (source, height, sinks),
/// where source is the highest points, height is its height, and sinks are the point connected to the source.
pub fn skeleton(polygon: &[Vec2], holes: &[&[Vec2]]) -> Vec<Subtree> {
    // println!("beginning skeleton {:?}", polygon);

    let mut vs = vec![];
    let mut lavs = vec![];
    let mut output = vec![];
    let mut queue = BinaryHeap::new();

    let mut slav = SLAV::new(&mut vs, &mut lavs, polygon, holes);

    for &lav in &slav.lavs {
        for vertex in lavs[lav.0].iter_keys(&vs) {
            if let Some(ev) = vs[vertex.0].next_event(&slav, &vs) {
                queue.push(Reverse(ev))
            }
        }
    }

    while !queue.is_empty() && !slav.lavs.is_empty() {
        //// println!("slav is {:?}", slav.lavs);

        let i = queue.pop().unwrap().0;
        // println!("managing event {:?}", i);

        let (arc, events) = match i {
            Event::Edge(e) => {
                if !vs[e.vertex_a.0].valid || !vs[e.vertex_b.0].valid {
                    // println!("{} Discarded outdated event {:?}", e.distance, e);
                    continue;
                }
                slav.handle_edge_event(&mut vs, &mut lavs, e)
            }
            Event::Split(s) => {
                if !vs[s.vertex.0].valid {
                    // println!("{} Discarded outdated event {:?}", s.distance, s);
                    continue;
                }
                slav.handle_split_event(&mut vs, &mut lavs, s)
            }
        };

        queue.extend(events.into_iter().map(Reverse));
        output.extend(arc.into_iter());
    }

    merge_sources(&mut output);

    output
}

pub fn faces_from_skeleton(poly: &[Vec2], skeleton: &[Subtree]) -> Vec<Vec<Vec3>> {
    let poly = normalize_contour(poly);
    let mut graph: HashMap<Vec2, Vec<_>> = HashMap::new();
    let mut heights = HashMap::new();

    for (&prev, &p, _) in window(&poly) {
        graph.entry(p).or_default().push(prev);
        graph.entry(prev).or_default().push(p);
        heights.insert(p, 0.0);
    }

    for tree in skeleton {
        if tree.source.magnitude2() > 1e10 {
            return vec![];
        }
        heights.insert(tree.source, tree.height);
        for &v in &tree.sinks {
            if v.magnitude2() > 1e10 {
                return vec![];
            }
            graph.entry(tree.source).or_default().push(v);
            graph.entry(v).or_default().push(tree.source);
        }
    }

    for (&r, l) in &mut graph {
        l.sort_unstable_by_key(|&p| OrderedFloat((r - p).angle(Vec2::UNIT_X)));
    }

    let mut faces = vec![];
    let mut visited = HashSet::new();

    fn next_v(graph: &HashMap<Vec2, Vec<Vec2>>, cur: Vec2, next: Vec2) -> (Vec2, Vec2) {
        let l = &graph[&next];
        let i = l.iter().position(|&x| x == cur).unwrap();
        let prev = if i == 0 { l.len() - 1 } else { i - 1 };
        (next, l[prev])
    }

    fn explore(
        graph: &HashMap<Vec2, Vec<Vec2>>,
        visited: &mut HashSet<(Vec2, Vec2)>,
        heights: &HashMap<Vec2, f32>,
        start: Vec2,
        mut next: Vec2,
    ) -> Vec<Vec3> {
        let mut face = vec![start.z(heights[&start])];
        let mut cur = start;
        visited.insert((cur, next));
        while next != start {
            let (c, n) = next_v(graph, cur, next);
            cur = c;
            next = n;
            face.push(cur.z(heights[&cur]));
            visited.insert((cur, next));
        }
        face
    }

    for (&node, l) in &graph {
        for &edge in l {
            if !visited.contains(&(node, edge)) {
                let face = explore(&graph, &mut visited, &heights, node, edge);
                if face.iter().all(|v| v.z == 0.0) {
                    continue;
                }
                faces.push(face);
            }
        }
    }

    faces
}

#[cfg(test)]
mod tests {
    use crate::skeleton::{faces_from_skeleton, skeleton};
    use crate::vec2;
    use ordered_float::OrderedFloat;

    #[test]
    fn test_box() {
        let poly = &[
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 10.0),
            vec2(0.0, 10.0),
        ];
        let skeleton = skeleton(poly, &[]);
        assert!(!skeleton.is_empty());
        let faces = faces_from_skeleton(poly, &skeleton);
        assert_eq!(faces.len(), 6);
    }

    #[test]
    fn test_big() {
        let mut skeleton = skeleton(
            &[
                vec2(208.0, 131.0),
                vec2(213.0, 142.0),
                vec2(168.0, 141.0),
                vec2(260.0, 168.0),
                vec2(246.0, 149.0),
                vec2(277.0, 142.0),
                vec2(271.0, 163.0),
                vec2(302.0, 180.0),
                vec2(268.0, 173.0),
                vec2(305.0, 196.0),
                vec2(319.0, 225.0),
                vec2(367.0, 214.0),
                vec2(423.0, 169.0),
                vec2(471.0, 160.0),
                vec2(540.0, 208.0),
                vec2(588.0, 268.0),
                vec2(616.0, 270.0),
                vec2(644.0, 308.0),
                vec2(630.0, 446.0),
                vec2(647.0, 472.0),
                vec2(641.0, 459.0),
                vec2(656.0, 467.0),
                vec2(660.0, 450.0),
                vec2(646.0, 423.0),
                vec2(687.0, 447.0),
                vec2(666.0, 495.0),
                vec2(651.0, 495.0),
                vec2(711.0, 580.0),
                vec2(728.0, 584.0),
                vec2(714.0, 557.0),
                vec2(746.0, 560.0),
                vec2(735.0, 569.0),
                vec2(744.0, 617.0),
                vec2(769.0, 594.0),
                vec2(753.0, 624.0),
                vec2(771.0, 628.0),
                vec2(793.0, 700.0),
                vec2(842.0, 708.0),
                vec2(871.0, 759.0),
                vec2(902.0, 780.0),
                vec2(891.0, 788.0),
                vec2(871.0, 773.0),
                vec2(887.0, 799.0),
                vec2(947.0, 774.0),
                vec2(964.0, 782.0),
                vec2(978.0, 689.0),
                vec2(985.0, 678.0),
                vec2(990.0, 695.0),
                vec2(984.0, 555.0),
                vec2(868.0, 338.0),
                vec2(854.0, 294.0),
                vec2(869.0, 316.0),
                vec2(887.0, 314.0),
                vec2(892.0, 366.0),
                vec2(895.0, 322.0),
                vec2(805.0, 196.0),
                vec2(747.0, 61.0),
                vec2(759.0, 59.0),
                vec2(753.0, 43.0),
                vec2(691.0, 33.0),
                vec2(683.0, 98.0),
                vec2(661.0, 72.0),
                vec2(355.0, 83.0),
                vec2(333.0, 46.0),
                vec2(35.0, 70.0),
                vec2(70.0, 144.0),
                vec2(50.0, 165.0),
                vec2(77.0, 154.0),
                vec2(87.0, 125.0),
                vec2(99.0, 139.0),
                vec2(106.0, 118.0),
                vec2(122.0, 139.0),
                vec2(89.0, 152.0),
                vec2(169.0, 124.0),
            ],
            &[],
        );

        assert!(!skeleton.is_empty());
        skeleton.sort_by_key(|x| OrderedFloat(x.source.x));
        for sub in skeleton {
            println!("{:?}", sub);
        }
    }
}
