;window.tab_langs = (function(){

	// find and group the pre/code elements with the matching languages
	// return [[e]]
	function find_groups(langs) {
		let groups = [];
		document.querySelectorAll("code").forEach(function(code){
			let lang = langs.find(
				lang => code.className.toLowerCase().split(/[ -]/).includes(lang.toLowerCase())
			);
			if (!lang) return;
			let pre = code.parentElement;
			pre.classList.add("tabbed");
			let last_group = groups[groups.length-1];
			let item = { lang, pre };
			if (last_group) {
				if (last_group[last_group.length-1].pre == pre.previousElementSibling) {
					last_group.push(item);
					return;
				}
			}
			groups.push([item]);
		});
		return groups;
	}

	function add_tabs(group) {
		let tabs = document.createElement("div");
		tabs.className = "lang-tabs";

		group.forEach((item, idx) => {
			let tab = document.createElement("span");
			tab.className = "lang-tab";
			tab.textContent = item.lang;
			tabs.appendChild(tab);

			if (idx) item.pre.style.display = "none";
			else tab.classList.add("active");

			tab.addEventListener("click", function(){
				tabs.querySelectorAll(".lang-tab").forEach(t => t.classList.remove("active"));
				group.forEach((_item, _idx) => {
					if (_idx == idx) _item.pre.style.display = "";
					else _item.pre.style.display = "none";
				});
				this.classList.add("active");
			});
		});

		group[0].pre.parentNode.insertBefore(tabs, group[0].pre);
	}

	// group together with tabs the provided languages
	function group(...langs) {
		let groups = find_groups(langs);
		for (let group of groups) {
			add_tabs(group);
		}
	}

	return {
		find_groups,
		add_tabs,
		group,
	};

})();
