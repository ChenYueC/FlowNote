from PIL import Image

img = Image.open("D:/ChenYue/Desktop/noteImg/output.png").convert("RGBA")

img.save(
    "D:/ChenYue/Desktop/FlowNote/src-tauri/icons/icon.ico",
    format="ICO",
    sizes=[
        (32, 32),
        (48, 48),
        (64, 64),
        (128, 128),
        (256, 256)
    ]
)