import json
import os

os.makedirs('restored_code', exist_ok=True)
files = {}

with open('C:/Users/jmach/.gemini/antigravity/brain/dececc29-df61-4327-8be6-6a1ed9cbed04/.system_generated/logs/transcript.jsonl', encoding='utf-8') as f:
    for line in f:
        step = json.loads(line)
        for call in step.get('tool_calls', []):
            if call['name'] == 'write_to_file':
                args = call['args']
                path = args.get('TargetFile', '').strip('"\'')
                if 'tauri-app' in path:
                    files[path] = args.get('CodeContent', '')
            elif call['name'] == 'replace_file_content':
                args = call['args']
                path = args.get('TargetFile', '').strip('"\'')
                if path in files:
                    files[path] += '\n// REPLACED:\n' + args.get('ReplacementContent', '')

for k, v in files.items():
    basename = os.path.basename(k)
    with open(f'restored_code/{basename}', 'w', encoding='utf-8') as out:
        out.write(v)
print('Done extracting files.')
