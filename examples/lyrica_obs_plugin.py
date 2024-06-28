# Lyrica OBS Plugin example.
# Reference: https://obsproject.com/forum/threads/help-updating-text-file-read-rate-to-be-faster.171431/

import obspython as obs
from typing import Optional
import threading
import asyncio
import websockets
import json


_LOOP: Optional[asyncio.AbstractEventLoop] = None
_THREAD: Optional[threading.Thread] = None


interval: str = 50
source_name: str = ""

metadata: str = ""
lyric_line: str = ""

# ------------------------------------------------------------

def update_text():
    global source_name

    source = obs.obs_get_source_by_name(source_name)
    if source is not None:

        settings = obs.obs_data_create()
        obs.obs_data_set_string(settings, "text", metadata + "\n" + lyric_line)
        obs.obs_source_update(source, settings)
        obs.obs_data_release(settings)

        obs.obs_source_release(source)

def refresh_pressed(props, prop):
    update_text()

# ------------------------------------------------------------

def script_description():
    return "Lyrica OBS Plugin."

def lyrica_thread():
    global metadata, lyric_line
    async def listen():
        global metadata, lyric_line
        async with websockets.connect("ws://127.0.0.1:15649/ws") as websocket:
            while True:
                message = json.loads(await websocket.recv())
                match message["id"]:
                    case 0:
                        metadata = "Now Playing: " + message["data"]["music_info"]["artist"] + " - " + message["data"]["music_info"]["title"]
                        lyric_line = ""
                    case 1:
                        lyric_line = message["data"]["lyric_line"]["lyric"]
    _LOOP = asyncio.new_event_loop()
    asyncio.set_event_loop(_LOOP)

    # Start anything that needs the async loop. The call to run_forever may
    # not be needed, depends on the way the loop dependent components are
    # started.
    _LOOP.run_until_complete(listen())

    # Stop anything that is running on the loop before closing. Most likely
    # using the loop run_until_complete function
    _LOOP.close()
    _LOOP = None



def script_load(settings):
    _THREAD = threading.Thread(None, lyrica_thread, daemon=True)
    _THREAD.start()

def script_unload():
    if _LOOP is not None:
        _LOOP.call_soon_threadsafe(lambda l: l.stop(), _LOOP)

    if _THREAD is not None:
        # Wait for 5 seconds, if it doesn't exit just move on not to block
        # OBS main thread. Logging something about the failure to properly exit
        # is advised.
        _THREAD.join(timeout=5)
        _THREAD = None

def script_update(settings):
    global interval
    global source_name

    interval = obs.obs_data_get_int(settings, "interval")
    source_name = obs.obs_data_get_string(settings, "source")

    if source_name != "":
        obs.timer_add(update_text, interval)

def script_defaults(settings):
    obs.obs_data_set_default_int(settings, "interval", 1000)

def script_properties():
    props = obs.obs_properties_create()

    obs.obs_properties_add_int(props, "interval", "Update Interval (Milliseconds)", 1, 3600, 1)

    p = obs.obs_properties_add_list(props, "source", "Text Source", obs.OBS_COMBO_TYPE_EDITABLE, obs.OBS_COMBO_FORMAT_STRING)
    sources = obs.obs_enum_sources()
    if sources is not None:
        for source in sources:
            source_id = obs.obs_source_get_unversioned_id(source)
            if source_id in ["text_gdiplus", "text_ft2_source"]:
                name = obs.obs_source_get_name(source)
                obs.obs_property_list_add_string(p, name, name)

        obs.source_list_release(sources)

    obs.obs_properties_add_button(props, "button", "Refresh", refresh_pressed)
    return props
