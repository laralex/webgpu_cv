use std::collections::{HashMap, HashSet};
use std::task::Poll;
use std::pin::Pin;

use futures::Future;

use crate::image_loader::{self, TextureInfo};

pub struct AssetLoader {
   textures: HashMap<AssetGUID, TextureAsset>,
   textures_guids: HashMap<String, AssetGUID>,
   textures_loading: HashSet<AssetGUID>,
   free_guid: AssetGUID,
}

impl AssetLoader {
   pub fn new() -> Self {
      Self {
         textures: HashMap::new(),
         textures_guids: HashMap::new(),
         textures_loading: HashSet::new(),
         free_guid: AssetGUID(1),

      }
   }

   pub fn tick_loading(&mut self, cx: &mut std::task::Context<'_>) {
      if self.textures_loading.is_empty() {
         return;
      }
      let guid = self.textures_loading.iter().take(1).next().unwrap().to_owned();
      // TODO: self.textures_get_many !
      let asset = self.textures.get_mut(&guid)
         .expect("BUG in AssetLoader - self.textures_loading contains GUID of non-existing texture");
      if let TextureAsset::Loading(future) = asset {
         if let Poll::Ready(texture) = future.as_mut().poll(cx) {
            *asset = TextureAsset::Texture(texture);
            self.textures_loading.remove(&guid);
            log::warn!("AssetLoader texture loaded GUID:{} res:{:?}", guid.0, asset.dimensions());
         } else {
            log::warn!("AssetLoader texture in progress GUID:{}", guid.0);
         }
      }
   }

   pub fn load_texture(&mut self, image_path: String) -> AssetGUID {
      if !self.textures_guids.contains_key(&image_path) {
         let guid = self.free_guid;
         log::info!("Loading texture asset: {}, GUID={}", image_path, guid.0);
         self.textures_guids.insert(image_path.clone(), guid);
         let loading = Box::pin(image_loader::load_image_rgba8(image_path));
         self.textures.insert(guid, TextureAsset::Loading(loading));
         self.textures_loading.insert(guid);
         self.free_guid.0 += 1;
         guid
      } else {
         self.textures_guids[&image_path]
      }
   }

   pub fn unload_texture(&mut self, guid: AssetGUID) {
      self.textures.remove(&guid);
   }

   pub fn get_texture(&mut self, guid: AssetGUID) -> Option<&TextureAsset> {
      self.textures.get(&guid)
   }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct AssetGUID(usize);
pub enum TextureAsset {
   Texture(TextureInfo),
   TextureLod(TextureInfo),
   Loading(Pin<Box<dyn Future<Output=TextureInfo>>>),
}

impl TextureAsset {
   pub fn data(&self) -> &[u8] {
      match self {
         TextureAsset::Texture(info) => &info.data,
         TextureAsset::TextureLod(info) => &info.data,
         TextureAsset::Loading(_) => &[255_u8, 0, 255, 255],
      }
   }

   pub fn dimensions(&self) -> (u32, u32, u32) {
      match self {
         TextureAsset::Texture(info) => (info.width, info.height, info.depth),
         TextureAsset::TextureLod(info) => (info.width, info.height, info.depth),
         TextureAsset::Loading(_) => (1, 1, 1),
      }
   }

   pub fn pixel_stride(&self) -> u8 {
      match self {
         TextureAsset::Texture(info) => info.pixel_stride,
         TextureAsset::TextureLod(info) => info.pixel_stride,
         TextureAsset::Loading(_) => 1,
      }
   }
}