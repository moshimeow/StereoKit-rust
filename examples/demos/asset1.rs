use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};
use stereokit_rust::{
    font::Font,
    include_asset_tree,
    material::Material,
    maths::{Matrix, Pose, Quat, Vec2, Vec3},
    mesh::Mesh,
    model::{AnimMode, Model},
    sk::{IStepper, SkInfo, StepperAction, StepperId},
    sprite::Sprite,
    system::{Assets, Handed, Input, Log, Renderer, Text, TextStyle},
    tex::Tex,
    tools::os_api::{get_assets, PathEntry},
    ui::{Ui, UiBtnLayout},
    util::named_colors::RED,
};

const ASSET_DIR: &[&str] = include_asset_tree!("assets");

pub struct Asset1 {
    id: StepperId,
    sk_info: Option<Rc<RefCell<SkInfo>>>,
    pub transform: Matrix,
    pub asset_pose: Pose,
    pub asset_scale: Vec3,
    model_to_show: Option<Model>,
    asset_files: Vec<PathEntry>,
    asset_sub_dir: PathBuf,
    exts: Vec<String>,
    pub window_pose: Pose,
    asset_selected: u32,
    radio_off: Sprite,
    radio_on: Sprite,
    text: String,
    text_style: TextStyle,
}

impl Default for Asset1 {
    fn default() -> Self {
        Self {
            id: "Asset1".to_string(),
            sk_info: None,
            transform: Matrix::tr(&((Vec3::NEG_Z * 2.5) + Vec3::Y), &Quat::from_angles(0.0, 180.0, 0.0)),
            asset_pose: Pose::new(Vec3::new(0.0, 1.3, -0.3), None),
            asset_scale: Vec3::ONE * 0.02,
            model_to_show: None,
            asset_files: vec![],
            asset_sub_dir: PathBuf::new(),
            exts: vec![".sks".into(), ".jpeg".into(), ".hdr".into(), ".glb".into(), ".gltf".into()],
            window_pose: Pose::new(Vec3::new(0.5, 1.5, -0.5), Some(Quat::from_angles(0.0, 180.0, 0.0))),
            asset_selected: 0,
            radio_off: Sprite::radio_off(),
            radio_on: Sprite::radio_on(),
            text: "Model1".to_owned(),
            text_style: Text::make_style(Font::default(), 0.3, RED),
        }
    }
}

impl IStepper for Asset1 {
    fn initialize(&mut self, id: StepperId, sk: Rc<RefCell<SkInfo>>) -> bool {
        self.id = id;
        self.asset_files = get_assets(sk.clone(), self.asset_sub_dir.clone(), &self.exts);
        self.sk_info = Some(sk);

        // Some test about hand meshes
        let left_hand = Input::get_controller_model(Handed::Left);
        let right_hand = Input::get_controller_model(Handed::Right);
        Input::set_controller_model(Handed::Left, Some(left_hand));
        Input::set_controller_model(Handed::Right, Some(right_hand));
        let material_hand = Material::unlit();
        Input::hand_material(Handed::Right, Some(material_hand));

        Log::diag(format!("{:?}", ASSET_DIR));

        true
    }

    fn step(&mut self, _event_report: &[StepperAction]) {
        self.draw()
    }
}

impl Asset1 {
    fn draw(&mut self) {
        // If a model has been selected, we draw it
        if let Some(model) = &self.model_to_show {
            Ui::handle("Model1", &mut self.asset_pose, model.get_bounds() * self.asset_scale, false, None, None);
            let model_transform = self.asset_pose.to_matrix(Some(self.asset_scale));
            Renderer::add_model(model, model_transform, None, None);
        } else {
            self.asset_selected = 0;
        }

        let mut new_asset_file = None;

        // The window to select existing model in this crate
        let window_text = if self.exts.is_empty() {
            format!("Assets/{:?}", self.asset_sub_dir)
        } else {
            format!("Assets/{:?} with type {:?}", self.asset_sub_dir, self.exts)
        };
        Ui::window_begin(window_text, &mut self.window_pose, Some(Vec2::new(0.5, 0.0)), None, None);

        let mut i = 0;
        for file_name in &self.asset_files {
            i += 1;

            if let PathEntry::File(name) = file_name {
                let file_name_str = name.to_str().unwrap_or("OsString error!!");
                Ui::same_line();
                if Ui::radio_img(
                    file_name_str,
                    self.asset_selected == i,
                    &self.radio_off,
                    &self.radio_on,
                    UiBtnLayout::Left,
                    None,
                ) {
                    self.model_to_show = open_asset(name, &self.asset_sub_dir, file_name_str);
                    self.asset_selected = i;
                }
            }
        }
        Ui::next_line();
        if let Some(sub_dir_name) = self.asset_sub_dir.to_str() {
            if !sub_dir_name.is_empty() {
                //---back button
                if Ui::button("..", None) {
                    self.asset_sub_dir.pop();
                    new_asset_file = Some(get_assets(
                        self.sk_info.as_ref().unwrap().clone(),
                        self.asset_sub_dir.clone(),
                        &self.exts,
                    ));
                }
            }
        }
        let cur_dir = self.asset_sub_dir.clone();
        // we add the dir at the end
        let mut sub_dir: String = cur_dir.to_string_lossy().to_string();
        if !sub_dir.is_empty() {
            sub_dir += "/";
        }
        let sub_asset_dir = "assets/".to_string() + &sub_dir;
        for dir_name_str in ASSET_DIR {
            if dir_name_str.starts_with(&sub_asset_dir) && dir_name_str.len() > sub_asset_dir.len() + 1 {
                let split_pos =
                    dir_name_str.char_indices().nth_back(dir_name_str.len() - sub_asset_dir.len() - 1).unwrap().0;
                let name = &dir_name_str[split_pos..];
                if !name.contains('/') {
                    Ui::same_line();
                    if Ui::button(name, None) {
                        self.asset_sub_dir.push(name);
                        new_asset_file = Some(get_assets(
                            self.sk_info.as_ref().unwrap().clone(),
                            self.asset_sub_dir.clone(),
                            &self.exts,
                        ));
                    }
                }
            }
        }

        if let Some(new_value) = new_asset_file {
            self.asset_files = new_value;
            self.asset_selected = 0;
        }
        Ui::window_end();

        Text::add_at(&self.text, self.transform, Some(self.text_style), None, None, None, None, None, None);
    }
}

/// Open asset regarding its extension
fn open_asset(name: &std::ffi::OsString, asset_sub_dir: &Path, file_name_str: &str) -> Option<Model> {
    let file_path = asset_sub_dir.join(name);
    if let Some(ext) = file_path.extension() {
        let ext = ".".to_string() + ext.to_str().unwrap_or("!!ERROR!!");
        if Assets::MODEL_FORMATS.contains(&ext.as_str()) {
            if let Ok(model) = Model::from_file(name, None) {
                let mut anims = model.get_anims();
                if anims.get_count() > 0 {
                    anims.play_anim_idx(0, AnimMode::Loop);
                }
                Some(model)
            } else {
                Log::err(format!("Unable to load model {:?} !!", file_name_str));
                None
            }
        } else if Assets::TEXTURE_FORMATS.contains(&ext.as_str()) {
            let model = Model::new();
            let mesh = Mesh::generate_plane_up(Vec2::ONE * 6.0, None, true);
            let tex = Tex::from_file(file_path, true, None).unwrap_or_default();
            let mut material = Material::default_copy();
            material.diffuse_tex(tex);
            model.get_nodes().add("tex_plane", Matrix::IDENTITY, mesh, material, true);
            Some(model)
        } else if ext == ".sks" {
            let model = Model::new();
            let mesh = Mesh::generate_plane_up(Vec2::ONE * 6.0, None, true);
            let tex = Tex::from_file("textures/open_gltf.jpeg", true, None).unwrap_or_default();
            if let Ok(mut material) = Material::from_file(&file_path, None) {
                material.diffuse_tex(tex);
                model.get_nodes().add("tex_plane", Matrix::IDENTITY, mesh, material, true);
                Some(model)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}