use oxidx_std::*;
use std::sync::{Arc, Mutex};

/// Mock state to track component layout results.
#[derive(Default, Debug)]
struct MockState {
    position: Vec2,
    size: Vec2,
    layout_calls: usize,
}

/// A mock component that records its position and size during layout.
struct MockComponent {
    id: String,
    state: Arc<Mutex<MockState>>,
    intrinsic_size: Vec2,
}

impl MockComponent {
    fn new(id: &str, width: f32, height: f32) -> (Self, Arc<Mutex<MockState>>) {
        let state = Arc::new(Mutex::new(MockState {
            position: Vec2::ZERO,
            size: Vec2::new(width, height),
            layout_calls: 0,
        }));
        (
            Self {
                id: id.to_string(),
                state: Arc::clone(&state),
                intrinsic_size: Vec2::new(width, height),
            },
            state,
        )
    }
}

impl OxidXComponent for MockComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn render(&self, _renderer: &mut Renderer) {}

    fn bounds(&self) -> Rect {
        let state = self.state.lock().unwrap();
        Rect::new(
            state.position.x,
            state.position.y,
            state.size.x,
            state.size.y,
        )
    }

    fn set_position(&mut self, x: f32, y: f32) {
        let mut state = self.state.lock().unwrap();
        state.position = Vec2::new(x, y);
    }

    fn set_size(&mut self, width: f32, height: f32) {
        let mut state = self.state.lock().unwrap();
        state.size = Vec2::new(width, height);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let mut state = self.state.lock().unwrap();
        state.layout_calls += 1;
        state.position = Vec2::new(available.x, available.y);
        state.size = self.intrinsic_size;
        self.intrinsic_size
    }
}

#[test]
fn test_vstack_layout_math() {
    let mut stack = VStack::with_spacing(Spacing::gap(10.0));

    let (c1, s1) = MockComponent::new("c1", 100.0, 100.0);
    let (c2, s2) = MockComponent::new("c2", 100.0, 100.0);
    let (c3, s3) = MockComponent::new("c3", 100.0, 100.0);

    stack.add_child(Box::new(c1));
    stack.add_child(Box::new(c2));
    stack.add_child(Box::new(c3));

    // Run layout
    let total_size = stack.layout(Rect::new(0.0, 0.0, 800.0, 600.0));

    // Assert container size: 3 * 100 (height) + 2 * 10 (gap) = 320
    assert_eq!(total_size.y, 320.0);
    assert_eq!(total_size.x, 100.0);

    // Assert child positions
    {
        let s = s1.lock().unwrap();
        assert_eq!(s.position.y, 0.0);
        assert_eq!(s.layout_calls, 1);
    }
    {
        let s = s2.lock().unwrap();
        assert_eq!(s.position.y, 110.0); // 100 + 10 gap
    }
    {
        let s = s3.lock().unwrap();
        assert_eq!(s.position.y, 220.0); // 110 + 100 + 10 gap
    }
}
