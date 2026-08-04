import os
import sys
import subprocess
from core.downloader import get_wwiser_path

def run_wwiser(wwise_path, txtp_dir, log_callback=None):
    wwiser_py = get_wwiser_path()
    python_exe = sys.executable
    
    if log_callback: log_callback("Scanning for .bnk files...")
    
    # 1. Flatten the Media folder so all localized .wem files are in the root Media directory
    media_dir = os.path.join(wwise_path, "Media")
    if os.path.exists(media_dir):
        if log_callback: log_callback("Flattening localized Media files...")
        import shutil
        for item in os.listdir(media_dir):
            sub_dir = os.path.join(media_dir, item)
            if os.path.isdir(sub_dir):
                for f in os.listdir(sub_dir):
                    if f.endswith('.wem'):
                        src = os.path.join(sub_dir, f)
                        dst = os.path.join(media_dir, f)
                        if not os.path.exists(dst):
                            shutil.copy2(src, dst)
    
    # 2. Find all .bnk files
    bnk_files = []
    for root, dirs, files in os.walk(wwise_path):
        for f in files:
            if f.endswith('.bnk'):
                bnk_files.append(os.path.join(root, f))
                
    if not bnk_files:
        if log_callback: log_callback("WRN: No .bnk files found. Skipping Wwiser extraction.")
        return False
        
    # 3. Split bnk files into chunks for parallel processing
    num_threads = 4
    chunk_size = max(1, len(bnk_files) // num_threads)
    chunks = [bnk_files[i:i + chunk_size] for i in range(0, len(bnk_files), chunk_size)]
    
    if log_callback: log_callback(f"Found {len(bnk_files)} .bnk files. Running {len(chunks)} parallel wwiser workers to speed up extraction...")
    
    import concurrent.futures
    import threading
    
    log_lock = threading.Lock()
    
    def safe_log(msg):
        if log_callback:
            with log_lock:
                log_callback(msg)
                
    def run_wwiser_chunk(chunk_idx, bnk_chunk):
        config_path = os.path.join(txtp_dir, f"wwconfig_args_{chunk_idx}.txt")
        with open(config_path, "w", encoding="utf-8") as f:
            f.write("-g\n")
            f.write("-go\n")
            f.write(f'"{txtp_dir}"\n')
            f.write("-gw\n")
            f.write('"../Media"\n')
            for bnk in bnk_chunk:
                f.write(f'"{bnk}"\n')
                
        cmd = [python_exe, wwiser_py, config_path]
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
            universal_newlines=True
        )
        
        for line in iter(process.stdout.readline, ''):
            if line:
                pass # Suppress chunk logs to avoid spamming the UI, only report completion
                
        process.stdout.close()
        return process.wait()

    success = True
    with concurrent.futures.ThreadPoolExecutor(max_workers=num_threads) as executor:
        futures = {executor.submit(run_wwiser_chunk, idx, chunk): idx for idx, chunk in enumerate(chunks)}
        for future in concurrent.futures.as_completed(futures):
            idx = futures[future]
            try:
                ret = future.result()
                if ret != 0:
                    safe_log(f"ERR: Wwiser worker {idx} exited with code {ret}")
                    success = False
                else:
                    safe_log(f"Wwiser worker {idx} finished successfully.")
            except Exception as e:
                safe_log(f"ERR: Wwiser worker {idx} failed: {e}")
                success = False

    # Cleanup config files
    for idx in range(len(chunks)):
        cfg = os.path.join(txtp_dir, f"wwconfig_args_{idx}.txt")
        if os.path.exists(cfg):
            try: os.remove(cfg)
            except: pass
            
    if success:
        safe_log("Wwiser parallel extraction completed successfully.")
    return success
