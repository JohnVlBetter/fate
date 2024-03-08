use cgmath::{Deg, Matrix4};
use std::mem::size_of;
use std::path::Path;
use std::time::Instant;

use crate::{skybox::SkyboxModel, texture::{load_hdr_image, Texture}};