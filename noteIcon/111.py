from PIL import Image

def resize_image(input_path, output_path, size=(256, 256), format="PNG"):
    """
    使用 Lanczos 算法缩放图片，保留最佳画质。
    
    参数:
        input_path  - 输入图片路径
        output_path - 输出图片路径
        size        - 目标尺寸，默认 (256, 256)
        format      - 输出格式，支持 PNG / WEBP / JPEG
    """
    img = Image.open(input_path)

    # 保持宽高比，居中裁剪为正方形
    w, h = img.size
    min_side = min(w, h)
    left = (w - min_side) // 2
    top  = (h - min_side) // 2
    img = img.crop((left, top, left + min_side, top + min_side))

    # Lanczos 缩放
    img = img.resize(size, Image.LANCZOS)

    # 保存
    save_kwargs = {"optimize": True}
    if format.upper() == "JPEG":
        save_kwargs["quality"] = 88
        if img.mode == "RGBA":
            img = img.convert("RGB")
    elif format.upper() == "WEBP":
        save_kwargs["quality"] = 85

    img.save(output_path, format.upper(), **save_kwargs)
    print(f"完成：{output_path}  尺寸：{img.size}  格式：{format.upper()}")


if __name__ == "__main__":
    # ── 在这里修改路径和参数 ──────────────────────────
    INPUT  = "icon_chatGpt.png"        # 输入图片路径
    OUTPUT = "output_256.png"   # 输出图片路径
    SIZE   = (256, 256)         # 目标尺寸
    FORMAT = "PNG"              # PNG / WEBP / JPEG
    # ────────────────────────────────────────────────

    resize_image(INPUT, OUTPUT, SIZE, FORMAT)