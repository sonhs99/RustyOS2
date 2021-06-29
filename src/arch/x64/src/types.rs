#[repr(C, packed(1))]
pub struct Queue<T> {
    max_count: isize,

    buffer: *mut T,
    put_index: isize,
    get_index: isize,

    last_operation_put: bool
}

impl<T: Copy> Queue<T> {
    pub fn new(size: isize, buffer: &mut [T]) -> Self {
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

    pub fn enqueue(&mut self, data: &T) -> Result<(), ()> {
        if self.is_full() { return Err(()); }
        unsafe { self.buffer[self.put_index] = *data; }
        self.put_index = (self.put_index + 1) % self.max_count;
        Ok(()) 
    }

    pub fn dequeue(&mut self) -> Result<T, ()> {
        if self.is_empty() { return Err(()); }
        let data = unsafe {self.buffer[self.get_index]};
        self.get_index = (self.get_index + 1) % self.max_count;
        Ok(data) 
    }
}