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
        radius_int: i16,
        min_worldx: f32,
        min_worldy: f32,
        world_to_int: f32,
    }

    impl BBoxGenInt {
        /// Slow function
        pub fn new(radius: f32, world: Rect<f32>) -> Self {
            let world = world.grow(radius);
            let int_dim = (i16::MAX as i64 - i16::MIN as i64) as f32;

            let dimx = world.x.end - world.x.start;
            let dimy = world.y.end - world.y.start;

            //TODO or max?
            let dim=dimx.min(dimy);

            let world_to_int = int_dim / dim;

            let radius_int = (radius * world_to_int).ceil() as i16;
            
            let min_worldx = world.x.start;
            let min_worldy = world.y.start;

            BBoxGenInt {
                radius_int,
                min_worldx,
                min_worldy,
                world_to_int,
            }
        }
        
        /// Fast function
        #[inline(always)]
        pub fn generate_bbox(&self, [posx, posy]: [f32; 2]) -> Rect<i16> {
            let x_int = (((posx - self.min_worldx) * self.world_to_int).round()) as i16;
            let y_int = (((posy - self.min_worldy) * self.world_to_int).round()) as i16;

            Rect::new(
                x_int - self.radius_int,
                x_int + self.radius_int,
                y_int - self.radius_int,
                y_int + self.radius_int,
            )
        }
    }
}
