from math import cos, tan, log, pi, radians
import requests
from PIL import Image
from io import BytesIO
from os import makedirs, path
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
import random

# --- CONFIGURATION ---
# bounding box to download
CENTER_LAT: float = 0
CENTER_LON: float = 0
RADIUS_KM: float = 5
ZOOM: int = 18

# CARTO Positron (Light) map style
MAP_URLS: list[str] = [
	"https://a.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png",
	"https://b.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png",
	"https://c.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png",
	"https://d.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png"
]

# Output
OUTPUT_DIR: str = "map_tiles_bin"
MAX_WORKERS: int = 16

session = requests.Session()
session.headers.update({ "User-Agent": "RustBikeComputerProject" })


def calculate_bounding_box(lat: float, lon: float, radius_km: float) -> tuple[float, float, float, float]:
	# 1 deg latitude is ~111.32 km
	lat_delta: float = radius_km / 111.32

	# 1 deg long shrinks depending on the lat
	lon_delta: float = radius_km / (111.32 * cos(radians(lat)))

	lat_max: float = lat + lat_delta # North edge
	lat_min: float = lat - lat_delta # South edge
	lon_max: float = lon + lon_delta # East edge
	lon_min: float = lon - lon_delta # West edge

	return lat_max, lat_min, lon_max, lon_min


def deg2num(lat: float, lon: float, zoom: int) -> tuple[int, int]:
	lat_rad: float = radians(lat)
	n: int = 2 ** zoom

	xtile: int = int((lon + 180.0) / 360.0 * n)
	ytile: int = int((1.0 - log(tan(lat_rad) + (1 / cos(lat_rad))) / pi) / 2.0 * n)

	return xtile, ytile


def process_and_save_tile(z: int, x: int, y: int) -> bool:
	bin_path: str = f"{OUTPUT_DIR}/{z}/{x}"
	file_path: str = f"{bin_path}/{y}.bin"

	if path.exists(file_path):
		return True
	
	url = random.choice(MAP_URLS).format(z=z, x=x, y=y)

	try:
		response = session.get(url, timeout=10)

		if response.status_code == 200:
			img = Image.open(BytesIO(response.content))
			img_gray = img.convert('L')

			THRESHOLD: int = 227
			lut: list[int] = [255 if p > THRESHOLD else 0 for p in range(256)]
			img_binary = img_gray.point(lut, '1')

			makedirs(bin_path, exist_ok=True)
			with open(file_path, "wb") as file:
				file.write(img_binary.tobytes())
			return True

		elif response.status_code == 429:
			# 429 means "Too Many Requests". Sleep this thread to cool off.
			time.sleep(5.0)
			return False

		else:
			return False
	except Exception:
		return False


def confirm(text: str) -> bool:
	res: str = input(text).lower()
	return res == "y" or res == "yes"


def main():
	print(f"Calculation bounding box for {RADIUS_KM} km radius around {CENTER_LAT}, {CENTER_LON}...")

	lat_max, lat_min, lon_max, lon_min = calculate_bounding_box(CENTER_LAT, CENTER_LON, RADIUS_KM)

	x_start, y_start = deg2num(lat_max, lon_min, ZOOM)
	x_end, y_end = deg2num(lat_min, lon_max, ZOOM)

	total_files = (x_end - x_start + 1) * (y_end - y_start + 1)

	total_bytes = total_files * 8192
	total_mb = total_bytes / (1024 * 1024)

	print(f"Grid size: {x_end - x_start + 1} x {y_end - y_start + 1} tiles")
	print(f"Total files to download: {total_files}")
	print(f"Estimated storage required: {total_mb:.2f} mb")

	if not confirm("Would you like to proceed? "):
		print("Aborting")
		return
	
	tasks = [(ZOOM, x, y) for x in range(x_start, x_end + 1) for y in range(y_start, y_end + 1)]

	print(f"Spinning up {MAX_WORKERS} concurrent workers...")
	success_count = 0

	with ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
		futures = { executor.submit(process_and_save_tile, z, x, y): (z, x, y) for (z, x, y) in tasks }

		for future in as_completed(futures):
			if future.result():
				success_count += 1
			
			if success_count % 1000 == 0:
				print(f"Progress: {success_count}/{total_files} tiles saved.")

	print("\nBatch complete! If any files failed, just run again! (Skips existing files)")


if __name__ == "__main__":
	main()

