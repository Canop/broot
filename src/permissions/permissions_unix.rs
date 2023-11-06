use {
    fnv::FnvHashMap,
    once_cell::sync::Lazy,
    std::sync::Mutex,
};

pub fn supported() -> bool {
    true
}

pub fn user_name(uid: u32) -> String {
    static USERS_CACHE_MUTEX: Lazy<Mutex<FnvHashMap<u32, String>>> = Lazy::new(|| {
            Mutex::new(FnvHashMap::default())
    });
    let mut users_cache = USERS_CACHE_MUTEX.lock().unwrap();
    let name = users_cache
        .entry(uid)
        .or_insert_with(|| {
            uzers::get_user_by_uid(uid).map_or_else(
                || "????".to_string(),
                |u| u.name().to_string_lossy().to_string(),
            )
        });
    (*name).to_string()
}

pub fn group_name(gid: u32) -> String {
    static GROUPS_CACHE_MUTEX: Lazy<Mutex<FnvHashMap<u32, String>>> = Lazy::new(|| {
        Mutex::new(FnvHashMap::default())
    });
    let mut groups_cache = GROUPS_CACHE_MUTEX.lock().unwrap();
    let name = groups_cache
        .entry(gid)
        .or_insert_with(|| {
            uzers::get_group_by_gid(gid).map_or_else(
                || "????".to_string(),
                |u| u.name().to_string_lossy().to_string(),
            )
        });
    (*name).to_string()
}
