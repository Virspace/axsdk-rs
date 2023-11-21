pub struct AxRect
{
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32
}

#[no_mangle]
pub fn kb(value: i64) -> i64 {
    value * 1024
}

#[no_mangle]
pub fn mb(value: i64) -> i64 {
    kb(value) * 1024
}

#[no_mangle]
pub fn gb(value: i64) -> i64 {
    mb(value) * 1024
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kb_test() {
        let result = kb(4);
        assert_eq!(result, 4096);
    }

    #[test]
    fn mb_test() {
        let result = mb(2);
        assert_eq!(result, 2097152);
    }

    #[test]
    fn gb_test() {
        let result = gb(2);
        assert_eq!(result, 2147483648)
    }
}