use utils::t_matrix::TMatrix;

pub mod text;
pub mod triangle;
pub mod utils;


pub trait UIElement {
    fn render(&mut self, window_size: (i32, i32));
    fn get_window_transform() -> TMatrix;
}