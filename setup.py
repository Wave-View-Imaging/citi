from setuptools import setup
from setuptools_rust import Binding, RustExtension
import os


setup(
    name="citi",
    version="0.3.0",
    rust_extensions=[
        RustExtension(
            "citi.citi",
            binding=Binding.NoBinding
        )
    ],
    packages=["citi"],
    package_dir={'': os.path.join('ffi', 'python')},
    zip_safe=False,
)
