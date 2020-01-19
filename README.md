# ThavalonWeb
Web App for THavalon

Currently, packages/twisted/internet/asyncioreactor.py errors out on python3.8 on Windows. To fix this, open asyncioreactor.py and add import os to the import statements. Then, change the following from :

try:
    from asyncio import get_event_loop
except ImportError:
    raise ImportError("Requires asyncio.")

to the following:

try:
    from asyncio import get_event_loop, set_event_loop_policy, WindowsSelectorEventLoopPolicy
except ImportError:
    raise ImportError("Requires asyncio.")

if sys.platform == 'win32':
    set_event_loop_policy(WindowsSelectorEventLoopPolicy())

https://stackoverflow.com/questions/58908293/i-keep-getting-notimplementederror-error-when-starting-django-server
