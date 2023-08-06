use std::marker::PhantomData;

pub const API_URL: &'static str = "https://images-api.nasa.gov";

pub const BASE_URL: &'static str = "https://images-api.nasa.gov";

#[inline] pub fn search() -> String {
    format!("{}/search", BASE_URL)
}

#[inline] pub fn asset(nasa_id: &str) -> String {
    format!("{}/asset/{}", BASE_URL, nasa_id)
}

#[inline] pub fn metadata(nasa_id: &str) -> String {
    format!("{}/metadata/{}", BASE_URL, nasa_id)
}

#[inline] pub fn captions(nasa_id: &str) -> String {
    format!("{}/captions/{}", BASE_URL, nasa_id)
}

#[inline] pub fn album(album_name: &str) -> String {
    format!("{}/album/{}", BASE_URL, album_name)
}

pub enum MediaType {
    IMAGE, VIDEO, AUDIO
}

pub enum Endpoints {
    Search {
        q: Option<String>,
        center: Option<String>,
        description: Option<String>,
        description_508: Option<String>,
        keywords: Option<Vec<String>>,
        location: Option<String>,
        media_type: Option<MediaType>,
        nasa_id: Option<String>,
        page: Option<usize>,
        page_size: Option<usize>,
        photographer: Option<String>,
        secondary_creator: Option<String>,
        title: Option<String>,
        year_start: Option<String>,
        year_end: Option<String>,
    }
}

pub struct Link {
    href: String,
    prompt: String,
    rel: String
}

pub struct Data<T> {
    data: T,
}

pub struct SearchResult {
    center: String,
    date_created: String,
    description: String,
    keywords: Vec<String>,
    media_type: String,
    nasa_id: String,
    title: String,
}

pub struct Collection<T, U = Data<T>> {
    href: String,
    items: Vec<U>,
    links: Vec<Link>,
    // metadata: 
    version: String,
    _phantom: PhantomData<T>
}

type SearchResponse = Collection<SearchResult>;