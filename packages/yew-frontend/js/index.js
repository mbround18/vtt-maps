function updateReadmeAnchors() {
  setInterval(() => {
    document.querySelectorAll("#readme a[href]").forEach((link) => {
      link.setAttribute("target", "_blank");
      link.setAttribute("rel", "noopener noreferrer");
    });
  }, 10);
}
