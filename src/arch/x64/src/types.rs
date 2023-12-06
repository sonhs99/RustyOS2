#[repr(C, packed(1))]
pub struct StaticQueue<T: Copy + 'static> {
    max_count: usize,

    buffer: &'static mut [T],
    put_index: usize,
    get_index: usize,

    last_operation_put: bool
}

impl<T: Copy + 'static> StaticQueue<T> {
    pub fn new(size: usize, buffer: &'static mut [T]) -> Self {
        Self{
            max_count: size,
            buffer: buffer,
            put_index: 0,
            get_index: 0,
            last_operation_put: false
        }
    }

    pub fn is_full(&self) -> bool {
        (self.get_index == self.put_index) && self.last_operation_put
    }

    pub fn is_empty(&self) -> bool {
        (self.get_index == self.put_index) && !self.last_operation_put
    }

    pub fn enqueue(&mut self, data: &T) -> bool {
        if self.is_full() { return false; }
        self.buffer[self.put_index] = *data;
        self.put_index = (self.put_index + 1) % self.max_count;
        true
    }

    pub fn dequeue(&mut self) -> Result<T, ()> {
        if self.is_empty() { return Err(()); }
        let data = self.buffer[self.get_index];
        self.get_index = (self.get_index + 1) % self.max_count;
        Ok(data) 
    }
}

unsafe impl<T: Copy + 'static> Sync for StaticQueue<T>{}