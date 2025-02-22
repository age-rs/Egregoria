macro_rules! gen_z_2 {
    {$($a: ident,)*;$($b: ident,)*} => {
        $(
            pub const $a: f32 = $b + 0.05;
        )+
    }
}

macro_rules! gen_z {
    {$a: ident $($v:ident)*} => {
        pub const $a: f32 = 0.01;
        gen_z_2!{$($v,)*MAX_Z,;$a,$($v,)*}
    }
}

gen_z! {
    Z_TERRAIN
    Z_GRID
    Z_LOT
    Z_BSPRITE
    Z_INTER_BG
    Z_LANE_BG
    Z_LANE
    Z_SIDEWALK
    Z_ARROW
    Z_CROSSWALK
    Z_HIGHLIGHT_INTER
    Z_GUITURN
    Z_SIGNAL
    Z_CAR
    Z_PATH_NOT_FOUND
    Z_DEBUG_BG
    Z_DEBUG
    Z_TOOL_BG
    Z_TOOL
}
