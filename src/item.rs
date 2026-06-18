pub struct Item {
    pub height: u32,
    pub width: u32,
    pub client: u32,
}

impl Item {
    fn new(height: u32, width: u32, client: u32) -> Self {
        Item {
            height,
            width,
            client,
        }
    }
}
