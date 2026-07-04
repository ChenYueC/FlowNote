from PIL import Image

img = Image.open("output_256.png").convert("RGBA")

new_data = []
for r, g, b, a in img.getdata():
    if r < 50 and g < 50 and b < 50:
        new_data.append((0, 0, 0, 0))
    else:
        new_data.append((r, g, b, a))

img.putdata(new_data)

# 用 LANCZOS 正确缩放（关键点）
img = img.resize(img.size, Image.Resampling.LANCZOS)

img.save("output.png")