use crate::foundation::axtypes::*;

pub fn get_rect_width(rect: AxRect) -> f32 {
    rect.right - rect.left
}

pub fn get_rect_height(rect: AxRect) -> f32 {
    rect.bottom - rect.top
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axrect_test() {
        let rect = AxRect {left: 2f32, top: 4f32, right: 6f32, bottom: 8f32};
        let result = get_rect_width(rect);

        assert_eq!(result, 4f32);
    }
}