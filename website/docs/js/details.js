/// Make sections with class "note details" collapsible
// (initially closed)
//
// Such a section is created by the following markdown:
//
// ```markdown
// !!! Note Details
//
//     Some **markdown** content
// ```
(function main(){
	$(".note.details").each(function(){
		let section = this;
		section.classList.add("closed");
		$(section).children(".admonition-title")
			.text("Details")
			.click(function(){
				section.classList.toggle("closed");
			});
	});
})();

