use crate::gui_elements::utils::t_matrix::TMatrix;

struct UITriangle {
    vertices: [[f32; 3]; 3],
    window_transform: TMatrix,
}