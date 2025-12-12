window.addEventListener("load", function(){

    // Join code blocks with the same set of languages
    // then add tabs to switch between them
    tab_langs.group("Hjson", "JSON", "TOML");

    // Highlight all code blocks
    hljs.registerAliases(["Hjson", "hjson"], { languageName: "rust" });
    hljs.highlightAll();

});

