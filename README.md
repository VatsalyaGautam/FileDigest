```
███████╗██╗██╗     ███████╗██████╗ ██╗ ██████╗ ███████╗███████╗████████╗
██╔════╝██║██║     ██╔════╝██╔══██╗██║██╔════╝ ██╔════╝██╔════╝╚══██╔══╝
█████╗  ██║██║     █████╗  ██║  ██║██║██║  ███╗█████╗  ███████╗   ██║   
██╔══╝  ██║██║     ██╔══╝  ██║  ██║██║██║   ██║██╔══╝  ╚════██║   ██║   
██║     ██║███████╗███████╗██████╔╝██║╚██████╔╝███████╗███████║   ██║   
╚═╝     ╚═╝╚══════╝╚══════╝╚═════╝ ╚═╝ ╚═════╝ ╚══════╝╚══════╝   ╚═╝   
```
# 🖇️ FileDigest

A Multithreaded File Hasher with a Real-time Terminal UI

FileDigest is a powerful file hashing tool written in Rust that makes computing file checksums fast and easy. It can process multiple files at once using parallel processing, and shows you live progress right in your terminal as it works through your files.

⸻

## 📌 Key Features

	•	⚡ Parallel hashing using worker threads
	•	🗂️ Recursive directory traversal (optional)
	•	🖥️ Live TUI showing Pending → Working → Done/Error
	•	🧵 Configurable worker count
	•	🔐 BLAKE3 hashing (fast, secure, incremental)
	•	⚠️ Strong validation for invalid paths, symlinks, sockets, FIFOs, devices, etc.
	•	🧱 Robust error model (thiserror + anyhow)
	•	📦 Cross-platform (Linux/macOS/Windows)

⸻

## 🏗️ High-Level Architecture

![Architecture Diagram](docs/flow_diagram.png)

⸻

## 🔄 Code Flow Explained

Here's what happens behind the scenes when you run the program:

### 1. CLI Parsing (cli.rs)

First, the program reads your input to understand what you want:
	•	Which files or folders you want to hash
	•	How many threads to use (--threads)
	•	Whether to dive into subdirectories (--recursive)

Before doing any actual work, it makes sure everything looks good by:
	•	Checking that all paths are valid
	•	Removing any duplicates you might have accidentally included
	•	Filtering out things that can't be hashed (like symlinks, special system files, or files you don't have permission to read)

What you get:
A clean, verified list of files that are ready to be processed.

⸻

### 2. File Discovery (jobs.rs)

Next, the program examines each path you provided:
	•	If it's a file → adds it directly to the work queue
	•	If it's a directory → scans through it to find all the files inside (going deeper into subdirectories if you asked for that)

For every actual file it finds, it creates a FileRecord that tracks its journey:

Pending -> Working -> Done / Error

These records get sent to:
	•	The job queue where workers will pick them up
	•	The display interface so you can watch the progress

⸻

### 3. Worker Thread Pool (jobs.rs)

The program then creates a team of worker threads (think of them as parallel workers all doing the same job):

Each worker follows a simple routine:
	1.	Grabs the next file from the queue
	2.	Updates its status to "Working"
	3.	Calculates the hash using BLAKE3, piece by piece
	4.	Reports back with either:
	•	Success and the computed hash
	•	An error message if something went wrong

These workers run completely independently, which means they can all process different files at the same time without waiting for each other.

⸻

### 4. File Hashing Engine (file_hash.rs)

When a worker hashes a file, here's what happens:
	•	Opens the file
	•	Reads it in manageable chunks (not all at once)
	•	Feeds each chunk into the BLAKE3 hasher
	•	Produces the final hash as a readable hexadecimal string

This chunk-by-chunk approach means even huge files won't overwhelm your computer's memory.

⸻

### 5. Real-Time TUI (tui.rs)

While all this is happening, the main program keeps the display updated.

It's constantly watching for:
	•	Updates from workers about their progress
	•	Your keyboard input (in case you want to quit with 'q' or Ctrl+C)

Every time something changes:
	•	The display updates to show a file's new status:
	•	Pending → Working
	•	Working → Done
	•	Working → Error

The display closes automatically when:
	•	Every file has been processed, and
	•	All workers have finished their jobs

⸻

### 6. Graceful Shutdown

Once everything's complete:
	•	Workers naturally finish up (since there's no more work)
	•	The display closes cleanly
	•	Your terminal returns to normal

And you're done!

⸻

## 📁 Project Structure

```
src/
├── main.rs         # Entry point: CLI -> jobs -> TUI orchestration
├── lib.rs          # Exposes main functionality for tests
├── cli.rs          # All clap argument parsing + validation
├── jobs.rs         # Worker threads, job/status channels, file discovery
├── file_hash.rs    # BLAKE3 incremental hashing
├── tui.rs          # ratatui-based UI for real-time updates
├── utils.rs        # Helper utilities (e.g., truncate_middle)
└── error.rs        # Custom error types (thiserror)
```

⸻

## 🧠 Module Responsibilities (What Each Part Does)

### main.rs
	•	Reads and interprets your command-line input
	•	Gathers all the file paths
	•	Sets up the list of files to process
	•	Launches the worker threads
	•	Keeps the live display running
	•	Waits for all workers to finish

### cli.rs
	•	Defines what arguments you can use
	•	Makes sure your inputs are valid
	•	Blocks anything that shouldn't be hashed

### jobs.rs
	•	Finds all the files using directory scanning
	•	Sets up communication channels between workers and the main program
	•	Contains the logic that workers follow
	•	Sends progress updates to the display

### file_hash.rs
	•	Opens and reads files
	•	Computes the BLAKE3 hash piece by piece
	•	Returns the hash as a hex string

### tui.rs
	•	Takes control of your terminal
	•	Draws the live results table
	•	Updates rows as files get processed
	•	Handles when you want to exit

### error.rs
	•	Provides clear, specific error types
	•	Makes it easy to handle and report problems

⸻

## 🚀 Getting Started

Want to try it out? Here's how to get FileDigest running on your machine:

### 1. Clone the Repository

First, grab a copy of the project:

```bash
git clone https://github.com/yourusername/filedigest.git
cd filedigest
```

Or if you want to make your own modifications, fork it first on GitHub and then clone your fork.

### 2. Create a Test File

Let's create a simple test file in the project directory:

```bash
echo "Hello, FileDigest!" > myfile.txt
```

### 3. Run It!

Now you can hash your file using cargo:

```bash
cargo run -- myfile.txt
```

That's it! You'll see the live terminal interface showing your file being processed and its hash being computed.

⸻

## ⚙️ Usage Examples

Once you've built the project, here are different ways to use it:

### Hash a single file

cargo run -- ./example.txt

Or if you've installed it:

filedigest ./example.txt

### Hash a folder with 4 threads

cargo run -- ./folder -t 4

### Hash without going into subdirectories

cargo run -- ./folder --no-recursive

### Hash multiple files at once

cargo run -- file1.txt file2.txt folder/


⸻

## 🔧 Dependencies
	•	blake3 — cryptographic hashing
	•	walkdir — directory traversal
	•	crossbeam-channel — fast MPMC channels
	•	crossterm — keyboard I/O & terminal control
	•	ratatui — TUI rendering
	•	clap — CLI parsing
	•	thiserror + anyhow — error handling

⸻

## 🧪 Tests

Run all tests

cargo test

Integration tests use tempfile to avoid touching real filesystem.

⸻

## 🚀 Future Improvements
	•	Output results into JSON/CSV
	•	Support SHA-256, SHA-512
	•	Hash verification mode
	•	File-type filters
	•	Progress percentage per-file

⸻