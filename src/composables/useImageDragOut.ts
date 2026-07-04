import { convertFileSrc } from "@tauri-apps/api/core";

export function useImageDragOut() {
  function startImageDrag(imagePath: string, event: DragEvent) {
    if (!event.dataTransfer) return;

    // Convert local path to a file-like drag
    // On Windows/Tauri, we provide the file path for FileDrop
    event.dataTransfer.setData("DownloadURL", `image/png:image.png:file:///${imagePath.replace(/\\/g, "/")}`);
    event.dataTransfer.setData("text/uri-list", `file:///${imagePath.replace(/\\/g, "/")}`);
    event.dataTransfer.setData("text/plain", imagePath);
    event.dataTransfer.effectAllowed = "copy";

    // Create a drag image
    const img = new Image();
    img.src = `https://asset.localhost/${encodeURIComponent(imagePath)}`;
    img.onload = () => {
      event.dataTransfer?.setDragImage(img, 40, 40);
    };
    // Fallback: small translucent element
    const ghost = document.createElement("div");
    ghost.style.cssText =
      "position:fixed;top:-100px;width:80px;height:60px;background:rgba(255,255,255,0.15);border-radius:8px;display:flex;align-items:center;justify-content:center;";
    // Use DOM API instead of innerHTML to avoid XSS risk
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("width", "24");
    svg.setAttribute("height", "24");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "white");
    svg.setAttribute("stroke-width", "2");
    const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
    rect.setAttribute("x", "3"); rect.setAttribute("y", "3");
    rect.setAttribute("width", "18"); rect.setAttribute("height", "18");
    rect.setAttribute("rx", "2");
    const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
    circle.setAttribute("cx", "8.5"); circle.setAttribute("cy", "8.5"); circle.setAttribute("r", "1.5");
    const polyline = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
    polyline.setAttribute("points", "21 15 16 10 5 21");
    svg.appendChild(rect);
    svg.appendChild(circle);
    svg.appendChild(polyline);
    ghost.appendChild(svg);
    document.body.appendChild(ghost);
    event.dataTransfer.setDragImage(ghost, 40, 30);
    setTimeout(() => ghost.remove(), 0);
  }

  /**
   * Convert a Tauri asset path (AppData/assets/xxx.png) to a
   * web-compatible URL that can be loaded in an <img> tag.
   */
  function assetPathToUrl(path: string): string {
    // Tauri v2: use convertFileSrc for local files
    try {
      return convertFileSrc(path);
    } catch {
      return `file:///${path.replace(/\\/g, "/")}`;
    }
  }

  return {
    startImageDrag,
    assetPathToUrl,
  };
}
