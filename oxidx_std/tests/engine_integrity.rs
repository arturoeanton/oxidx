use oxidx_core::*;
use oxidx_std::*;
use std::sync::{Arc, Mutex};

/// Mock component to track event reception.
struct MockComponent {
    bounds: Rect,
    id: String,
    handled_return: bool,
    received: Arc<Mutex<bool>>,
}

impl MockComponent {
    fn new(id: &str, bounds: Rect, handled_return: bool) -> (Self, Arc<Mutex<bool>>) {
        let r = Arc::new(Mutex::new(false));
        (
            Self {
                bounds,
                id: id.to_string(),
                handled_return,
                received: Arc::clone(&r),
            },
            r,
        )
    }
}

impl OxidXComponent for MockComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn render(&self, _renderer: &mut Renderer) {}

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
    }

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        let mut r = self.received.lock().unwrap();
        *r = true;
        self.handled_return
    }
}

#[test]
fn test_z_order_hit_testing() {
    // 1. Initialize headless WGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: true,
    }));

    let adapter: wgpu::Adapter = match adapter {
        Some(a) => a,
        None => {
            println!("No software adapter available, limiting test scope");
            return;
        }
    };

    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
            .expect("Failed to create device");

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    // 2. Create headless context
    let mut ctx =
        OxidXContext::new_headless(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 800, 600);

    // 3. Setup overlapping components
    let mut zstack = ZStack::new();

    // Component A: Background (Bottom)
    let (c_bg, r_bg) = MockComponent::new("bg", Rect::new(0.0, 0.0, 500.0, 500.0), true);

    // Component B: Button (Top)
    let (c_btn, r_btn) = MockComponent::new("btn", Rect::new(10.0, 10.0, 100.0, 100.0), true);

    zstack.add_child(Box::new(c_bg));
    zstack.add_child(Box::new(c_btn)); // Last added is on top in ZStack

    // 4. Simulate a MouseDown at (20, 20) -> Inside BOTH, but BTN is on top
    let event = OxidXEvent::MouseDown {
        button: MouseButton::Left,
        position: Vec2::new(20.0, 20.0),
        modifiers: Modifiers::default(),
    };

    let result = zstack.on_event(&event, &mut ctx);

    // Assertions
    assert!(result, "ZStack should report event as handled");
    assert!(
        *r_btn.lock().unwrap(),
        "Top component (btn) should have received the event"
    );
    assert!(
        !*r_bg.lock().unwrap(),
        "Bottom component (bg) should NOT have received the event (propagation stopped)"
    );

    // 5. Test another event outside BTN but inside BG
    {
        // Reset received flags
        *r_btn.lock().unwrap() = false;
        *r_bg.lock().unwrap() = false;

        let event_bg = OxidXEvent::MouseDown {
            button: MouseButton::Left,
            position: Vec2::new(200.0, 200.0), // Outside btn (10,10,100,100)
            modifiers: Modifiers::default(),
        };

        let result_bg = zstack.on_event(&event_bg, &mut ctx);

        assert!(result_bg, "ZStack should report event as handled");
        assert!(
            !*r_btn.lock().unwrap(),
            "Top component should NOT have received event at 200,200"
        );
        assert!(
            *r_bg.lock().unwrap(),
            "Bottom component (bg) should have received the event at 200,200"
        );
    }
}
