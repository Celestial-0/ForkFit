import subprocess
import sys
import os

def run_codegen():
    # Ensure generated directory exists
    os.makedirs("./src/generated", exist_ok=True)
    
    # Run the grpcio-tools protoc compiler
    subprocess.check_call([
        sys.executable, "-m", "grpc_tools.protoc",
        "-I../proto/intelligence/v1",
        "--python_out=./src/generated",
        "--pyi_out=./src/generated",
        "--grpc_python_out=./src/generated",
        "../proto/intelligence/v1/intelligence.proto"
    ])
    
    # Fix import paths in compiled stubs
    path = "./src/generated/intelligence_pb2_grpc.py"
    if os.path.exists(path):
        with open(path, "r") as f:
            text = f.read()
        with open(path, "w") as f:
            f.write(text.replace("import intelligence_pb2", "from . import intelligence_pb2"))
        print("Protobuf stubs successfully generated and import path fixed in:", path)
    else:
        print(f"Error: Generated stub not found at {path}")

if __name__ == "__main__":
    run_codegen()
