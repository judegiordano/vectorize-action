without rayon 1173.058744152s

withj rayon no sql writes: 596.088604395s

544.173506952s

518.332749703s
552.654620966s

current iteration 538.365687682s

current iteratio 493.589682218s

    #[inline]
    pub fn is_excluded(&self, path: &str) -> bool {
        self.inputs.excludes.iter().any(|skip| path.contains(skip))
    }