document.getElementById("loadEnv").addEventListener("click", () => {
  const output = document.getElementById("output");
  fetch("/env")
    .then((response) => response.json())
    .then((data) => {
      output.innerHTML = "";
      const treeView = document.createElement("div");
      treeView.className = "treeview";
      createTreeView(data, treeView);
      output.appendChild(treeView);
    })
    .catch((error) => {
      console.error("Error loading environment variables:", error);
      alert("Failed to load environment variables.");
    });
});

function createTreeView(obj, parent) {
  const ul = document.createElement("ul");
  for (const key in obj) {
    const li = document.createElement("li");
    const span = document.createElement("span");
    span.textContent = key + ": " + obj[key];
    li.appendChild(span);
    ul.appendChild(li);
    if (typeof obj[key] === "object" && obj[key] !== null) {
      createTreeView(obj[key], li);
    }
  }
  parent.appendChild(ul);
}
