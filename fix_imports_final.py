import os
import re

def fix_file(path, crate_context):
    with open(path, 'r') as f:
        content = f.read()

    new_content = content
    if crate_context == 'gui':
        # gui uses mouser_engine
        new_content = re.sub(r'\bengine::lock_ext::MutexExt', 'mouser_engine::lock_ext::MutexExt', new_content)
    elif crate_context == 'engine':
        # engine uses crate::
        new_content = re.sub(r'\bengine::lock_ext::MutexExt', 'crate::lock_ext::MutexExt', new_content)
        new_content = re.sub(r'\bmouser_engine::lock_ext::MutexExt', 'crate::lock_ext::MutexExt', new_content)
    elif crate_context == 'root':
        # root uses engine::
        new_content = re.sub(r'\bmouser_engine::lock_ext::MutexExt', 'engine::lock_ext::MutexExt', new_content)

    if new_content != content:
        with open(path, 'w') as f:
            f.write(new_content)
        print(f"Fixed {path}")

for root, _, files in os.walk('.'):
    if 'target' in root or '.git' in root:
        continue
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            if path.startswith('./gui/src'):
                fix_file(path, 'gui')
            elif path.startswith('./engine/src'):
                fix_file(path, 'engine')
            elif path.startswith('./src'):
                fix_file(path, 'root')
            elif path.startswith('./engine/tests'):
                # tests usually use the crate name
                fix_file(path, 'root') # treating as root since it uses engine::
