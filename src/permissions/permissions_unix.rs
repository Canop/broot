use {
    once_cell::sync::Lazy,
    rustc_hash::FxHashMap,
    std::sync::Mutex,
};

pub fn supported() -> bool {
    true
}

pub fn user_name(uid: u32) -> String {
    static USERS_CACHE_MUTEX: Lazy<Mutex<FxHashMap<u32, String>>> =
        Lazy::new(|| Mutex::new(FxHashMap::default()));
    let mut users_cache = USERS_CACHE_MUTEX.lock().unwrap();
    let name = users_cache.entry(uid).or_insert_with(|| {
        // when the name can't be resolved (eg on statically linked musl
        // builds, which have no NSS), we fall back to the numeric id,
        // like `ls -n` does
        uzers::get_user_by_uid(uid).map_or_else(
            || uid.to_string(),
            |u| u.name().to_string_lossy().to_string(),
        )
    });
    (*name).clone()
}

pub fn group_name(gid: u32) -> String {
    static GROUPS_CACHE_MUTEX: Lazy<Mutex<FxHashMap<u32, String>>> =
        Lazy::new(|| Mutex::new(FxHashMap::default()));
    let mut groups_cache = GROUPS_CACHE_MUTEX.lock().unwrap();
    let name = groups_cache.entry(gid).or_insert_with(|| {
        // same fallback as in `user_name`
        uzers::get_group_by_gid(gid).map_or_else(
            || gid.to_string(),
            |u| u.name().to_string_lossy().to_string(),
        )
    });
    (*name).clone()
}
