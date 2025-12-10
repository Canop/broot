window.addEventListener("load", function(){

    // gather together group of code blocks with the same set of languages
    // then add tabs to switch between them
    let groups = tab_langs.find_groups(["Hjson", "JSON", "TOML"]);
    for (let group of groups) {
        tab_langs.add_tabs(group);
    }

    // highlight all code blocks
    hljs.registerAliases(["Hjson", "hjson"], { languageName: "rust" });
    hljs.highlightAll();

});

