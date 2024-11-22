# Murkmaw

Murkmaw is a Rust-based multithreaded web crawler designed for efficient link graph construction, image extraction, and customizable logging. It features a modular architecture that supports future enhancements and customization.

---

## Features

### Multithreaded Web Crawler
- **Parallel Crawling:** Utilizes multithreading for faster page scraping with configurable worker threads.
- **Link Graph Construction:** Maintains a graph structure (`LinkGraph`) tracking parent-child associations and link references.
- **Data Extraction:** Retrieves links, images, and titles from web pages.
- **Customizable Crawling:** Specify the maximum number of links and images to process.

### Enhanced Logging
- **Progress Bars:** Displays link discovery progress with a real-time progress bar.
- **Spinners:** Visual feedback for different stages of image processing and serialization.
- **Customizable Output:** Built using the `indicatif` and `console` crates.

### Image Utilities
- **Metadata Handling:** Converts extracted links into image metadata, including alt text and source URL.
- **Image Downloading:** Saves images locally in a user-defined directory.
- **Image Database:** Serializes image metadata into a JSON database.


## Getting Started
### Prerequisites
- Rust (latest stable version)
- Crates used in the project:
- tokio (for asynchronous operations)
- reqwest (for HTTP requests)
- serde and serde_json (for serialization and JSON handling)
- rayon (for multithreading)
- indicatif and console (for logging and UI enhancements)
- anyhow (for error handling)

## Installation
Clone the repository:

   ```bash
   git clone https://github.com/IAmFarrokhnejad/Murkmaw.git
    cd Murkmaw
```
Install dependencies:
 ```bash
   cargo build

```
## Usage
Run the application with the following command:
 ```bash
cargo run --release -- --starting_url <URL> --max_links <N> --max_images <N> --n_worker_threads <N> --log_status <true/false> --img_save_dir <directory> --links_json <filename>

```

## Command-Line Options
- starting_url: The initial URL to crawl (required).
- max_links: The maximum number of links to process (default: 100).
- max_images: The maximum number of images to extract (default: 50).
- n_worker_threads: Number of worker threads for parallel crawling (default: 4).
- log_status: Whether to enable logging (default: true).
- img_save_dir: Directory to save downloaded images (default: ./images).
- links_json: Filename for the JSON file storing the link graph (default: links.json).


## Contribution Guidelines
Contributions are welcome! Please follow these steps:
1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Submit a pull request with a clear description of your changes.


## License
This project is licensed under the MIT License - see the LICENSE file for details.