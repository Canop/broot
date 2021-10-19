
(function ltd_main(){
	function updateLink() {
		$(".link-to-dystroy").remove();
		let $brand = $(".navbar-brand");
		let available = $brand.offset().left;
		if (available < 30) return;
		$("<a>")
			.attr("href", "https://dystroy.org")
			.addClass("link-to-dystroy")
			.prependTo(".navbar");
	}
	window.addEventListener("resize", updateLink);
	updateLink();
})();


