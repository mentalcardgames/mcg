use std::cell::RefCell;
use std::rc::Rc;
use eframe::epaint::TextureHandle;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, MediaStreamConstraints,
};

type CameraFacing = web_sys::VideoFacingModeEnum;

pub struct Camera {
    state: Rc<RefCell<CameraState>>,
    frame_texture: Option<TextureHandle>,
    facing_mode: CameraFacing,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            state: Rc::new(RefCell::new(CameraState::Uninitialized)),
            frame_texture: None,
            facing_mode: CameraFacing::Environment,
        }
    }
}

enum CameraState {
    Uninitialized,
    Initializing(Rc<()>),
    Active(CameraSession),
}

struct CameraSession {
    video: HtmlVideoElement,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    stream: web_sys::MediaStream,
}

impl CameraSession {
    fn stop(self) {
        let tracks = self.stream.get_tracks();
        for i in 0..tracks.length() {
            if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                track.stop();
            }
        }
        self.video.set_src_object(None);
    }

    async fn start(facing_mode: CameraFacing) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window");
        let document = window.document().expect("no document");
        let video = document
            .create_element("video")?
            .dyn_into::<HtmlVideoElement>()?;
        video.set_autoplay(true);
        video.set_muted(true);
        if let Err(e) = video.set_attribute("playsinline", "true") {
            crate::sprintln!("Failed to set playsinline attribute: {:?}", e);
        }
        video.set_width(640);
        video.set_height(480);
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        canvas.set_width(640);
        canvas.set_height(480);
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;
        let navigator = window.navigator();
        let media_devices = navigator
            .media_devices()
            .map_err(|_| JsValue::from_str("MediaDevices not available"))?;
        let constraints = MediaStreamConstraints::new();
        let video_constraints = js_sys::Object::new();
        js_sys::Reflect::set(
            &video_constraints,
            &JsValue::from_str("facingMode"),
            &JsValue::from(facing_mode),
        )?;
        constraints.set_video(&video_constraints.into());
        let stream_promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream = wasm_bindgen_futures::JsFuture::from(stream_promise).await?;
        let media_stream = stream.dyn_into::<web_sys::MediaStream>()?;
        video.set_src_object(Some(&media_stream));
        let video_clone = video.clone();
        if let Ok(play_promise) = video_clone.play() {
            let _ = wasm_bindgen_futures::JsFuture::from(play_promise).await;
        }

        Ok(CameraSession {
            video: video_clone,
            canvas,
            context,
            stream: media_stream,
        })
    }
}

impl Camera {
    pub fn start(&mut self) {
        let token = Rc::new(());
        let previous_state = self
            .state
            .replace(CameraState::Initializing(token.clone()));
        if let CameraState::Active(session) = previous_state {
            session.stop();
        }

        let facing_mode = self.facing_mode;
        let state = self.state.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = CameraSession::start(facing_mode).await;
            Self::finish_start(&state, &token, result);
        });
    }

    fn finish_start(
        state: &RefCell<CameraState>,
        token: &Rc<()>,
        result: Result<CameraSession, JsValue>,
    ) {
        let mut current_state = state.borrow_mut();
        let is_current_start = matches!(
            &*current_state,
            CameraState::Initializing(current_token) if Rc::ptr_eq(current_token, token)
        );

        if !is_current_start {
            drop(current_state);
            if let Ok(session) = result {
                session.stop();
            }
            return;
        }

        *current_state = match result {
            Ok(session) => CameraState::Active(session),
            Err(_) => CameraState::Uninitialized,
        };
    }
    pub fn stop(&mut self) {
        let previous_state = self.state.replace(CameraState::Uninitialized);
        if let CameraState::Active(session) = previous_state {
            session.stop()
        }
        self.frame_texture = None;
    }
    fn with_active_session<T>(
        &self,
        f: impl FnOnce(&CameraSession) -> Result<T, JsValue>,
    ) -> Result<Option<T>, JsValue> {
        let state = self.state.borrow();
        if let CameraState::Active(session) = &*state {
            f(session).map(Some)
        } else {
            Ok(None)
        }
    }
    pub fn capture_frame(
        &mut self,
        ctx: &egui::Context,
    ) -> Result<Option<egui::ColorImage>, JsValue> {
        let Some(frame_data) = self.with_active_session(|session| {
            let ready_state = session.video.ready_state();
            let video_width = session.video.video_width();
            let video_height = session.video.video_height();
            let paused = session.video.paused();
            let ended = session.video.ended();
            if ready_state < 2 || video_width == 0 || video_height == 0 || paused || ended {
                return Ok(None);
            }

            let canvas_width = video_width.min(640);
            let canvas_height = video_height.min(480);
            if session.canvas.width() != canvas_width || session.canvas.height() != canvas_height {
                session.canvas.set_width(canvas_width);
                session.canvas.set_height(canvas_height);
            }
            session
                .context
                .clear_rect(0.0, 0.0, canvas_width as f64, canvas_height as f64);
            session
                .context
                .draw_image_with_html_video_element_and_dw_and_dh(
                    &session.video,
                    0.0,
                    0.0,
                    canvas_width as f64,
                    canvas_height as f64,
                )?;
            let image_data = session.context.get_image_data(
                0.0,
                0.0,
                canvas_width as f64,
                canvas_height as f64,
            )?;
            Ok(Some((canvas_width, canvas_height, image_data.data())))
        })? else {
            return Ok(None);
        };

        let Some((canvas_width, canvas_height, data)) = frame_data else {
            return Ok(None);
        };
        if data.is_empty() {
            return Ok(None);
        }
        let mut pixels = Vec::with_capacity((canvas_width * canvas_height) as usize);
        for i in (0..data.len()).step_by(4) {
            if i + 3 < data.len() {
                let r = data[i];
                let g = data[i + 1];
                let b = data[i + 2];
                let a = 255;
                pixels.push(egui::Color32::from_rgba_unmultiplied(r, g, b, a));
            }
        }
        if pixels.len() != (canvas_width * canvas_height) as usize {
            return Ok(None);
        }
        let color_image =
            egui::ColorImage::new([canvas_width as usize, canvas_height as usize], pixels);
        if let Some(texture) = &mut self.frame_texture {
            texture.set(color_image.clone(), egui::TextureOptions::LINEAR);
        } else {
            self.frame_texture = Some(ctx.load_texture(
                "camera_frame",
                color_image.clone(),
                egui::TextureOptions::LINEAR,
            ));
        }
        Ok(Some(color_image))
    }
    pub fn get_texture(&self) -> Option<&TextureHandle> {
        self.frame_texture.as_ref()
    }
    pub fn is_active(&self) -> bool {
        matches!(&*self.state.borrow(), CameraState::Active(_))
    }
    pub fn flip_camera(&mut self) {
        match self.facing_mode {
            CameraFacing::User => self.facing_mode = CameraFacing::Environment,
            CameraFacing::Environment => self.facing_mode = CameraFacing::User,
            _ => (),
        };
    }
    pub fn get_facing_mode(&self) -> CameraFacing {
        self.facing_mode
    }
}
