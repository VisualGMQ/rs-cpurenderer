use crate::math;
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use std::ops::Not;
use std::str::{self, SplitWhitespace};

/// a help struct to read whole file in lines
struct FileContent {
    lines: Vec<String>,
}

impl FileContent {
    fn from_file(filename: &std::path::Path) -> Result<FileContent, std::io::Error> {
        let file = std::fs::File::open(filename)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let mut lines: Vec<String> = vec![];
        let mut read_finish = false;
        while !read_finish {
            match reader.read_line(&mut line) {
                Ok(len) => {
                    if len != 0 {
                        lines.push(line.clone());
                        line.clear();
                    } else {
                        read_finish = true;
                    }
                }
                Err(err) => return Err(err),
            };
        }

        Ok(FileContent { lines })
    }
}

// Some scene data structure

pub struct Vertex {
    pub vertex: u32,
    pub normal: Option<u32>,
    pub texcoord: Option<u32>,
}

pub struct Face {
    pub vertices: Vec<Vertex>,
}

pub struct Model {
    pub faces: Vec<Face>,
    pub name: String,
    pub mtllib: Option<u32>,
    pub material: Option<String>,
    pub smooth_shade: u8,
}

pub struct MtlTextureMaps {
    pub ambient: Option<String>,            // map_Ka
    pub diffuse: Option<String>,            // map_Kd
    pub specular_color: Option<String>,     // map_Ks
    pub specular_highlight: Option<String>, // map_Ns
    pub alpha: Option<String>,              // map_d
    pub refl: Option<String>,               // map_refl
}

pub struct Material {
    pub name: String,
    pub ambient: Option<math::Vec3>,             // Ka
    pub diffuse: Option<math::Vec3>,             // Kd
    pub specular: Option<math::Vec3>,            // Ks
    pub emissive_coeficient: Option<math::Vec3>, // Ke
    pub specular_exponent: Option<f32>,          // Ns
    pub dissolve: Option<f32>,                   // d, Tr (d = 1.0 - Tr)
    pub transmission_filter: Option<math::Vec3>, // Tf
    pub optical_density: Option<f32>,            // Ni
    pub illum: Option<u8>,                       // illum

    pub texture_maps: MtlTextureMaps,
}

impl Material {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ambient: None,
            diffuse: None,
            specular: None,
            emissive_coeficient: None,
            specular_exponent: None,
            dissolve: None,
            transmission_filter: None,
            optical_density: None,
            illum: None,
            texture_maps: MtlTextureMaps {
                ambient: None,
                diffuse: None,
                specular_color: None,
                specular_highlight: None,
                alpha: None,
                refl: None,
            },
        }
    }
}

pub struct Mtllib {
    pub materials: HashMap<String, Material>,
}

pub struct SceneData {
    pub vertices: Vec<math::Vec3>,
    pub normals: Vec<math::Vec3>,
    pub texcoords: Vec<math::Vec2>,
    pub materials: Vec<Mtllib>,
    pub models: Vec<Model>,
}

impl SceneData {
    fn new() -> Self {
        SceneData {
            vertices: vec![],
            normals: vec![],
            texcoords: vec![],
            materials: vec![],
            models: vec![],
        }
    }
}

// Parser

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    CantCvt2Num,
    UnknownToken(String),
    ExccedComponent,
    EmptyContent,
    ParseIncomplete,
    InvalidSyntax,
    PathNotFount,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

struct TokenRequester<'a> {
    content: &'a FileContent,
    tokens: SplitWhitespace<'a>,
    line: u64,
}

#[derive(PartialEq)]
enum TokenType<'a> {
    Token(&'a str),
    Nextline,
    Eof,
}

impl<'a> TokenRequester<'a> {
    fn new(content: &'a FileContent) -> Result<Self, Error> {
        if content.lines.is_empty() {
            Err(Error::EmptyContent)
        } else {
            Ok(Self {
                content,
                tokens: content.lines[0].split_whitespace(),
                line: 0,
            })
        }
    }

    fn request(&mut self) -> TokenType {
        match self.tokens.next() {
            Some(token) => TokenType::Token(token),
            None => {
                self.line += 1;
                if self.line as usize >= self.content.lines.len() {
                    TokenType::Eof
                } else {
                    self.tokens = self.content.lines[self.line as usize].split_whitespace();
                    TokenType::Nextline
                }
            }
        }
    }
}

pub type ParseResult = Result<(), Error>;

struct ObjParser<'a, 'b> {
    scene: SceneData,
    dirpath: &'a std::path::Path,
    token_requester: &'b mut TokenRequester<'b>,
}

macro_rules! ignore_until {
    ($var:ident = $op:expr ; $($cond:expr),+) => {
        while $($var != $cond &&)+ true {
            $var = $op;
        }
    };
}

macro_rules! parse_as {
    ($token:ident = $request:expr; $type:ty = $($component:ident : $parse_type:ty),+) => {
        {
            let mut value = <$type>::zero();

            $(
                $token = $request;
                if let TokenType::Token(content) = $token {
                    value.$component = content.parse::<$parse_type>().map_err(|_| Error::CantCvt2Num)?;
                } else {
                    $token = $request;
                    return Err(Error::ParseIncomplete);
                }
            )+

            $token = $request;
            Ok::<$type, Error>(value)
        }
    };
    ($token:ident = $request:expr; String) => {
        {
            $token = $request;
            let result = if let TokenType::Token(content) = $token {
                Ok(content.to_string())
            } else {
                Err(Error::ParseIncomplete)
            };
            $token = $request;
            result
        }
    };
    ($token:ident = $request:expr; $parse_type:ty) => {
        {
            $token = $request;
            let result = if let TokenType::Token(content) = $token {
                Ok(content.parse::<$parse_type>().map_err(|_| Error::CantCvt2Num)?)
            } else {
                Err(Error::ParseIncomplete)
            };
            $token = $request;
            result
        }
    };
}

impl<'a, 'b> ObjParser<'a, 'b> {
    fn new(path: &'a std::path::Path, token_requester: &'b mut TokenRequester<'b>) -> Self {
        Self {
            scene: SceneData::new(),
            dirpath: path,
            token_requester,
        }
    }

    fn parse(&mut self) -> ParseResult {
        let mut token = self.token_requester.request();

        let mut parse_finish = false;
        while !parse_finish {
            match token {
                TokenType::Token(token_str) => match token_str {
                    "#" => ignore_until![token = self.token_requester.request();
                                              TokenType::Nextline, TokenType::Eof],
                    "g" | "o" => self.scene.models.push(Model {
                        faces: vec![],
                        name: parse_as![token = self.token_requester.request(); String]?,
                        mtllib: self
                            .scene
                            .materials
                            .is_empty()
                            .not()
                            .then_some((self.scene.materials.len() - 1) as u32),
                        material: None,
                        smooth_shade: 0,
                    }),
                    "v" => {
                        self.scene
                            .vertices
                            .push(parse_as![token = self.token_requester.request();
                                                              math::Vec3 = x: f32, y: f32, z: f32]?)
                    }
                    "vt" => self
                        .scene
                        .texcoords
                        .push(parse_as![token = self.token_requester.request();
                                                                math::Vec2 = x: f32, y: f32]?),
                    "vn" => {
                        self.scene
                            .normals
                            .push(parse_as![token = self.token_requester.request();
                                                              math::Vec3 = x: f32, y: f32, z: f32]?)
                    }
                    "f" => {
                        token = self.token_requester.request();
                        let mut vertices: Vec<Vertex> = vec![];

                        let mut finish = false;
                        while !finish {
                            if let TokenType::Token(token_str) = token {
                                let indices: Vec<&str> = token_str.split('/').collect();
                                if indices.len() != 3 {
                                    return Err(Error::InvalidSyntax);
                                }
                                let vertex =
                                    indices[0].parse::<u32>().map_err(|_| Error::CantCvt2Num)? - 1;

                                let texcoord = if indices[1].is_empty() {
                                    None
                                } else {
                                    Some(
                                        indices[1]
                                            .parse::<u32>()
                                            .map_err(|_| Error::CantCvt2Num)?
                                            - 1,
                                    )
                                };
                                let normal = if indices[2].is_empty() {
                                    None
                                } else {
                                    Some(
                                        indices[2]
                                            .parse::<u32>()
                                            .map_err(|_| Error::CantCvt2Num)?
                                            - 1,
                                    )
                                };
                                vertices.push(Vertex {
                                    vertex,
                                    normal,
                                    texcoord,
                                });
                            } else {
                                finish = true;
                            }
                            token = self.token_requester.request();
                        }

                        self.scene
                            .models
                            .last_mut()
                            .ok_or(Error::ParseIncomplete)?
                            .faces
                            .push(Face { vertices });
                    }
                    "mtllib" => {
                        token = self.token_requester.request();
                        if let TokenType::Token(mtllib_filename) = token {
                            let mut pathbuf = std::path::PathBuf::from(
                                self.dirpath.parent().ok_or(Error::PathNotFount)?,
                            );
                            pathbuf.push(mtllib_filename);
                            let filecontent = FileContent::from_file(pathbuf.as_path())?;
                            let mut mtllib_token_requester = TokenRequester::new(&filecontent)?;
                            let mut mtllib_parser = MtllibParser::new(&mut mtllib_token_requester);

                            self.scene.materials.push(mtllib_parser.parse()?);

                            token = self.token_requester.request();
                        }
                    }
                    "usemtl" => {
                        self.scene
                            .models
                            .last_mut()
                            .ok_or(Error::ParseIncomplete)?
                            .material =
                            Some(parse_as![token = self.token_requester.request(); String]?)
                    }
                    "s" => {
                        self.scene
                            .models
                            .last_mut()
                            .ok_or(Error::ParseIncomplete)?
                            .smooth_shade = parse_as![token = self.token_requester.request(); u8]?
                    }
                    _ => return Err(Error::UnknownToken(token_str.to_string())),
                },
                TokenType::Eof => parse_finish = true,
                TokenType::Nextline => token = self.token_requester.request(),
            }
        }
        Ok(())
    }
}

struct MtllibParser<'a> {
    token_requester: &'a mut TokenRequester<'a>,
}

macro_rules! parse_material_field {
    ($mtl:ident.$($member:ident).+ = $parse_expr:expr) => {
        $mtl.as_mut().ok_or(Error::ParseIncomplete)?
        $(.$member)+ = $parse_expr
    };
}

impl<'a> MtllibParser<'a> {
    fn new(token_requester: &'a mut TokenRequester<'a>) -> MtllibParser<'a> {
        Self { token_requester }
    }

    fn parse(&mut self) -> Result<Mtllib, Error> {
        let mut mtllib = Mtllib {
            materials: HashMap::new(),
        };

        let mut mtl: Option<Material> = None;

        let mut token = self.token_requester.request();

        let mut finish = false;
        while !finish {
            match token {
                TokenType::Token(token_str) => match token_str {
                    "#" => ignore_until![token = self.token_requester.request();
                                             TokenType::Nextline, TokenType::Eof],
                    "newmtl" => {
                        if let Some(m) = mtl {
                            mtllib.materials.insert(m.name.clone(), m);
                        }
                        mtl = Some(Material::new(
                            &parse_as![token = self.token_requester.request(); String]?,
                        ));
                    }
                    "Ns" => parse_material_field![
                        mtl.specular_exponent =
                            Some(parse_as![token = self.token_requester.request(); f32]?)
                    ],
                    "Ka" => parse_material_field![
                        mtl.ambient = Some(
                            parse_as![token = self.token_requester.request(); math::Vec3 = x: f32, y: f32, z: f32]?
                        )
                    ],
                    "Kd" => parse_material_field![
                        mtl.diffuse = Some(
                            parse_as![token = self.token_requester.request(); math::Vec3 = x: f32, y: f32, z: f32]?
                        )
                    ],
                    "Ks" => parse_material_field![
                        mtl.specular = Some(
                            parse_as![token = self.token_requester.request(); math::Vec3 = x: f32, y: f32, z: f32]?
                        )
                    ],
                    "Ke" => parse_material_field![
                        mtl.emissive_coeficient = Some(
                            parse_as![token = self.token_requester.request(); math::Vec3 = x: f32, y: f32, z: f32]?
                        )
                    ],
                    "Tf" => parse_material_field![
                        mtl.transmission_filter = Some(
                            parse_as![token = self.token_requester.request(); math::Vec3 = x: f32, y: f32, z: f32]?
                        )
                    ],
                    "Ni" => parse_material_field![
                        mtl.optical_density =
                            Some(parse_as![token = self.token_requester.request(); f32]?)
                    ],
                    "d" => parse_material_field![
                        mtl.dissolve =
                            Some(parse_as![token = self.token_requester.request(); f32]?)
                    ],
                    "Tr" => parse_material_field![
                        mtl.dissolve =
                            Some(1.0 - parse_as![token = self.token_requester.request(); f32]?)
                    ],
                    "illum" => parse_material_field![
                        mtl.illum = Some(parse_as![token = self.token_requester.request(); u8]?)
                    ],
                    "map_Ka" => parse_material_field![
                        mtl.texture_maps.ambient =
                            Some(parse_as![token = self.token_requester.request(); String]?)
                    ],
                    "map_Kd" => parse_material_field![
                        mtl.texture_maps.diffuse =
                            Some(parse_as![token = self.token_requester.request(); String]?)
                    ],
                    "map_Ks" => parse_material_field![
                        mtl.texture_maps.specular_color =
                            Some(parse_as![token = self.token_requester.request(); String]?)
                    ],
                    "map_Ns" => parse_material_field![
                        mtl.texture_maps.specular_highlight =
                            Some(parse_as![token = self.token_requester.request(); String]?)
                    ],
                    "map_d" => parse_material_field![
                        mtl.texture_maps.alpha =
                            Some(parse_as![token = self.token_requester.request(); String]?)
                    ],
                    "map_refl" => parse_material_field![
                        mtl.texture_maps.refl =
                            Some(parse_as![token = self.token_requester.request(); String]?)
                    ],
                    _ => return Err(Error::UnknownToken(token_str.to_string())),
                },
                TokenType::Nextline => token = self.token_requester.request(),
                TokenType::Eof => {
                    if let Some(m) = mtl {
                        mtllib.materials.insert(m.name.clone(), m);
                        mtl = None;
                    }
                    finish = true;
                }
            }
        }

        Ok(mtllib)
    }
}

/// load scene from file
pub fn load_from_file(filename: &str) -> Result<SceneData, Error> {
    match FileContent::from_file(std::path::Path::new(filename)) {
        Ok(content) => {
            let mut token_requester = TokenRequester::new(&content)?;
            let mut parser = ObjParser::new(std::path::Path::new(filename), &mut token_requester);
            parser.parse()?;
            Ok(parser.scene)
        }
        Err(err) => Err(Error::IoError(err)),
    }
}
