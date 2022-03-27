
(function main(){
	let groups = find_groups(["Hjson", "JSON", "TOML"]);
	console.log("groups:", groups);
	for (let group of groups) {
		add_tabs(group);
	}
})();


// find and group the pre/code elements with the matching languages
// return [[e]]
function find_groups(langs) {
	let groups = [];
	$("code").each(function(){
		let lang = langs.find(
			lang => this.className.toLowerCase().split(/[ -]/).includes(lang.toLowerCase())
		);
		if (!lang) return;
		let pre = this.parentElement;
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
	let $tabs = $("<div class=lang-tabs>");
	group.forEach((item, idx) => {
		let $tab = $("<span class=lang-tab>").text(item.lang).appendTo($tabs);
		if (idx) $(item.pre).hide();
		else $tab.addClass("active");
		$tab.click(function(){
			$tabs.find(".lang-tab").removeClass("active");
			group.forEach((_item, _idx) => {
				if (_idx==idx) $(_item.pre).show();
				else $(_item.pre).hide();
			});
			this.classList.add("active");
		});
	});
	$tabs.insertBefore(group[0].pre);
}
