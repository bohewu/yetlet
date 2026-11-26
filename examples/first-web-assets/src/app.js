const message = document.getElementById("message");
const preview = document.getElementById("preview");
const previewText = document.getElementById("preview-text");

function showPreview() {
  previewText.textContent = message.value;
  preview.hidden = message.value.trim() === "";
}

message.addEventListener("input", showPreview);
showPreview();
