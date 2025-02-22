use crate::geometry::earcut::earcut;
use crate::{MeshBuilder, MeshVertex};
use common::Z_GRID;
use geom::{vec2, Intersect, LinearColor, Segment, Vec2, AABB};

pub struct Tesselator {
    pub color: LinearColor,
    pub meshbuilder: MeshBuilder,
    pub cull_rect: Option<AABB>,
    pub zoom: f32,
    pub normal: [f32; 3],
}

impl Tesselator {
    pub fn new(cull_rect: Option<AABB>, zoom: f32) -> Self {
        Tesselator {
            color: LinearColor::BLACK,
            meshbuilder: MeshBuilder::new(),
            cull_rect,
            zoom,
            normal: [0.0, 0.0, 1.0],
        }
    }
}

impl Tesselator {
    pub fn draw_circle(&mut self, p: Vec2, z: f32, r: f32) -> bool {
        if r <= 0.0 || self.cull_rect.map_or(false, |x| !x.contains_within(p, r)) {
            return false;
        }
        let n_points = ((6.0 * (r * self.zoom).cbrt()) as usize).max(4);

        self.draw_regular_polygon(p, z, r, n_points, 0.0)
    }

    pub fn draw_regular_polygon(
        &mut self,
        p: Vec2,
        z: f32,
        r: f32,
        n_points: usize,
        start_angle: f32,
    ) -> bool {
        if r <= 0.0 || self.cull_rect.map_or(false, |x| !x.contains_within(p, r)) {
            return false;
        }

        let color = self.color.into();
        let n_pointsu32 = n_points as u32;
        let normal = self.normal;

        self.meshbuilder.extend_with(|vertices, index_push| {
            vertices.push(MeshVertex {
                position: [p.x, p.y, z],
                color,
                normal,
                uv: [0.0; 2],
            });

            for i in 0..n_pointsu32 {
                let v = std::f32::consts::PI * 2.0 * (i as f32) / n_points as f32 + start_angle;
                let trans = p + r * vec2(v.cos(), v.sin());
                vertices.push(MeshVertex {
                    position: [trans.x, trans.y, z],
                    color,
                    normal,
                    uv: [0.0; 2],
                });
                index_push(0);
                index_push(i + 1);
                if i == n_pointsu32 - 1 {
                    index_push(1);
                } else {
                    index_push(i + 2);
                }
            }
        });

        true
    }

    pub fn draw_filled_polygon(&mut self, points: &[Vec2], z: f32) -> bool {
        let oob = self.cull_rect.map_or(false, |x| {
            !points.iter().any(|&p| x.contains_within(p, 1.0))
        });
        if oob {
            return false;
        }

        let color: [f32; 4] = self.color.into();
        let normal = self.normal;
        self.meshbuilder.extend_with(|vertices, index_push| {
            vertices.extend(points.iter().map(|p| MeshVertex {
                position: [p.x, p.y, z],
                color,
                normal,
                uv: [0.0; 2],
            }));

            // Safe because Vector2 and [f32; 2] have the same layout (Vector2 is repr(c))
            let points: &[[f32; 2]] = unsafe { &*(points as *const [Vec2] as *const [[f32; 2]]) };
            earcut(bytemuck::cast_slice(points), |x, y, z| {
                index_push(x as u32);
                index_push(y as u32);
                index_push(z as u32);
            });
        });

        true
    }

    pub fn draw_stroke_circle(&mut self, p: Vec2, z: f32, r: f32, thickness: f32) -> bool {
        if r <= 0.0 || self.cull_rect.map_or(false, |x| !x.contains_within(p, r)) {
            return false;
        }

        let halfthick = thickness * 0.5;
        let n_points = ((6.0 * (r * self.zoom).cbrt()) as usize).max(4);
        let n_pointsu32 = n_points as u32;

        let color = self.color.into();
        let normal = self.normal;
        self.meshbuilder.extend_with(|vertices, index_push| {
            vertices.push(MeshVertex {
                position: [p.x + r + halfthick, p.y, z],
                color,
                normal,
                uv: [0.0; 2],
            });
            vertices.push(MeshVertex {
                position: [p.x + r - halfthick, p.y, z],
                color,
                normal,
                uv: [0.0; 2],
            });

            for i in 0..n_pointsu32 {
                let v = std::f32::consts::PI * 2.0 * (i as f32) / n_points as f32;
                let trans = vec2(v.cos(), v.sin());
                let p1 = p + (r + halfthick) * trans;
                let p2 = p + (r - halfthick) * trans;
                vertices.push(MeshVertex {
                    position: [p1.x, p1.y, z],
                    color,
                    normal,
                    uv: [0.0; 2],
                });
                vertices.push(MeshVertex {
                    position: [p2.x, p2.y, z],
                    color,
                    normal,
                    uv: [0.0; 2],
                });
                index_push(i * 2 + 2);
                index_push(i * 2 + 1);
                index_push(i * 2);

                index_push(i * 2 + 1);
                index_push(i * 2 + 2);
                index_push(i * 2 + 3);
            }

            let i = n_pointsu32;

            index_push(0);
            index_push(i * 2 + 1);
            index_push(i * 2);

            index_push(i * 2 + 1);
            index_push(0);
            index_push(1);
        });
        true
    }

    pub fn set_color(&mut self, color: impl Into<LinearColor>) {
        self.color = color.into();
    }

    pub fn draw_rect_cos_sin(
        &mut self,
        p: Vec2,
        z: f32,
        width: f32,
        height: f32,
        cos_sin: Vec2,
    ) -> bool {
        if let Some(x) = self.cull_rect {
            if !x.contains_within(p, width.max(height)) {
                return false;
            }
        }

        let a = (width * 0.5) * cos_sin;
        let b = (height * 0.5) * vec2(-cos_sin.y, cos_sin.x);
        let pxy = vec2(p.x, p.y);

        let points: [Vec2; 4] = [a + b + pxy, a - b + pxy, -a - b + pxy, -a + b + pxy];

        let color: [f32; 4] = self.color.into();

        let verts: [MeshVertex; 4] = [
            MeshVertex {
                position: [points[0].x, points[0].y, z],
                color,
                normal: self.normal,
                uv: [0.0; 2],
            },
            MeshVertex {
                position: [points[1].x, points[1].y, z],
                color,
                normal: self.normal,
                uv: [0.0; 2],
            },
            MeshVertex {
                position: [points[2].x, points[2].y, z],
                color,
                normal: self.normal,
                uv: [0.0; 2],
            },
            MeshVertex {
                position: [points[3].x, points[3].y, z],
                color,
                normal: self.normal,
                uv: [0.0; 2],
            },
        ];
        self.meshbuilder.extend(&verts, &[0, 1, 2, 0, 2, 3]);
        true
    }

    pub fn draw_stroke(&mut self, p1: Vec2, p2: Vec2, z: f32, thickness: f32) -> bool {
        if let Some(x) = self.cull_rect {
            if !x.expand(thickness * 0.5).intersects(&Segment::new(p1, p2)) {
                return false;
            }
        }

        let diff = p2 - p1;
        let dist = diff.magnitude();
        if dist < 1e-5 {
            return false;
        }
        let ratio = (thickness * 0.5) / dist;
        let perp: Vec2 = ratio * diff.perpendicular();

        let points: [Vec2; 4] = [p1 - perp, p1 + perp, p2 + perp, p2 - perp];

        let color: [f32; 4] = self.color.into();

        let verts: [MeshVertex; 4] = [
            MeshVertex {
                position: [points[0].x, points[0].y, z],
                color,
                normal: self.normal,
                uv: [0.0; 2],
            },
            MeshVertex {
                position: [points[1].x, points[1].y, z],
                color,
                normal: self.normal,
                uv: [0.0; 2],
            },
            MeshVertex {
                position: [points[2].x, points[2].y, z],
                color,
                normal: self.normal,
                uv: [0.0; 2],
            },
            MeshVertex {
                position: [points[3].x, points[3].y, z],
                color,
                normal: self.normal,
                uv: [0.0; 2],
            },
        ];
        self.meshbuilder.extend(&verts, &[0, 1, 2, 0, 2, 3]);
        true
    }

    pub fn draw_polyline_with_dir(
        &mut self,
        points: &[Vec2],
        first_dir: Vec2,
        last_dir: Vec2,
        z: f32,
        thickness: f32,
    ) -> bool {
        let n_points = points.len();
        if n_points < 2 || thickness <= 0.0 {
            return true;
        }
        if n_points == 2 {
            self.draw_stroke(points[0], points[1], z, thickness);
            return true;
        }
        if let Some(cull_rect) = self.cull_rect {
            let window_intersects = |x: &[Vec2]| {
                cull_rect
                    .expand(thickness)
                    .intersects(&Segment::new(x[0], x[1]))
            };

            if !points.windows(2).any(window_intersects) {
                return false;
            }
        }

        let halfthick = thickness * 0.5;

        let color = self.color.into();
        let normal = self.normal;
        self.meshbuilder.extend_with(move |verts, index_push| {
            let nor: Vec2 = halfthick * vec2(-first_dir.y, first_dir.x);

            verts.push(MeshVertex {
                position: [points[0].x + nor.x, points[0].y + nor.y, z],
                color,
                normal,
                uv: [0.0; 2],
            });

            verts.push(MeshVertex {
                position: [points[0].x - nor.x, points[0].y - nor.y, z],
                color,
                normal,
                uv: [0.0; 2],
            });

            let mut index: u32 = 0;

            for window in points.windows(3) {
                let a = window[0];
                let elbow = window[1];
                let c = window[2];

                let ae = unwrap_or!((elbow - a).try_normalize(), continue);
                let ce = unwrap_or!((elbow - c).try_normalize(), continue);

                let mut dir = match (ae + ce).try_normalize() {
                    Some(x) => x,
                    None => -ae.perpendicular(),
                };

                if ae.perp_dot(ce) < 0.0 {
                    dir = -dir;
                }

                let mul = 1.0 + (1.0 + ae.dot(ce).min(0.0)) * (std::f32::consts::SQRT_2 - 1.0);

                let p1 = elbow + mul * dir * halfthick;
                let p2 = elbow - mul * dir * halfthick;
                verts.push(MeshVertex {
                    position: [p1.x, p1.y, z],
                    color,
                    normal,
                    uv: [0.0; 2],
                });
                verts.push(MeshVertex {
                    position: [p2.x, p2.y, z],
                    color,
                    normal,
                    uv: [0.0; 2],
                });

                index_push(index * 2);
                index_push(index * 2 + 1);
                index_push(index * 2 + 2);

                index_push(index * 2 + 3);
                index_push(index * 2 + 2);
                index_push(index * 2 + 1);

                index += 1;
            }

            let nor: Vec2 = halfthick * vec2(-last_dir.y, last_dir.x);

            let p1 = points[n_points - 1] + nor;
            let p2 = points[n_points - 1] - nor;
            verts.push(MeshVertex {
                position: [p1.x, p1.y, z],
                color,
                normal,
                uv: [0.0; 2],
            });
            verts.push(MeshVertex {
                position: [p2.x, p2.y, z],
                color,
                normal,
                uv: [0.0; 2],
            });

            index_push(index * 2);
            index_push(index * 2 + 1);
            index_push(index * 2 + 2);

            index_push(index * 2 + 3);
            index_push(index * 2 + 2);
            index_push(index * 2 + 1);
        });
        true
    }

    pub fn draw_polyline(&mut self, points: &[Vec2], z: f32, thickness: f32) -> bool {
        let n_points = points.len();
        if n_points < 2 || thickness <= 0.0 {
            return true;
        }
        if n_points == 2 {
            self.draw_stroke(points[0], points[1], z, thickness);
            return true;
        }
        let first_dir = (points[1] - points[0]).normalize();
        let n = points.len();
        let last_dir = (points[n - 1] - points[n - 2]).normalize();
        self.draw_polyline_with_dir(points, first_dir, last_dir, z, thickness)
    }

    pub fn draw_line(&mut self, p1: Vec2, p2: Vec2, z: f32) -> bool {
        self.draw_stroke(p1, p2, z, 1.5 / self.zoom)
    }

    pub fn draw_grid(&mut self, grid_size: f32) {
        let screen = self
            .cull_rect
            .expect("Cannot draw grid when not culling since I do not know where is the screen");

        let startx = (screen.ll.x / grid_size).ceil() * grid_size;
        for x in 0..(screen.w() / grid_size) as i32 {
            let x = startx + x as f32 * grid_size;
            self.draw_line(vec2(x, screen.ll.y), vec2(x, screen.ur.y), Z_GRID);
        }

        let starty = (screen.ll.y / grid_size).ceil() * grid_size;
        for y in 0..(screen.h() / grid_size) as i32 {
            let y = starty + y as f32 * grid_size;
            self.draw_line(vec2(screen.ll.x, y), vec2(screen.ur.x, y), Z_GRID);
        }
    }
}
