pub mod bbox {
    use axgeom::Rect;

    ///
    /// Instead of using f32 bbox, this will create a i16 bbox
    /// given the position of an object. This can have performance
    /// improvements because the whole bbox fits in 64 bits.
    ///
    /// The bbox has to be rounded to fit into i16. It will pick the
    /// smallest i16 bbox that would cover the f32 bbox.
    ///
    /// Since bboxes are used for broadphase which isnt exact anyway,
    /// we can afford to use a lower resolution number type.
    ///
    /// Use this when you have a bounded f32 world, filled with
    /// objects of the same radius.
    ///
    pub struct BBoxGenInt {
        radiusx_int: i16,
        radiusy_int: i16,
        min_worldx: f32,
        min_worldy: f32,
        world_to_intx: f32,
        world_to_inty: f32,
    }

    impl BBoxGenInt {
        /// Slow function
        pub fn new(radius: f32, world: Rect<f32>) -> Self {
            let world = world.grow(radius);
            let int_dim = (i16::MAX as i64 - i16::MIN as i64) as f32;

            let dimx = world.x.end - world.x.start;
            let dimy = world.y.end - world.y.start;

            let world_to_intx = int_dim / dimx;
            let world_to_inty = int_dim / dimy;

            let radiusx_int = (radius * world_to_intx).ceil() as i16;
            let radiusy_int = (radius * world_to_inty).ceil() as i16;

            let min_worldx = world.x.start;
            let min_worldy = world.y.start;

            BBoxGenInt {
                radiusx_int,
                radiusy_int,
                min_worldx,
                min_worldy,
                world_to_intx,
                world_to_inty,
            }
        }
        /// Fast function
        pub fn generate_bbox(&self, [posx, posy]: [f32; 2]) -> Rect<i16> {
            let x_int = (((posx - self.min_worldx) * self.world_to_intx).round()) as i16;
            let y_int = (((posy - self.min_worldy) * self.world_to_inty).round()) as i16;

            Rect::new(
                x_int - self.radiusx_int,
                x_int + self.radiusx_int,
                y_int - self.radiusy_int,
                y_int + self.radiusy_int,
            )
        }
    }
}
