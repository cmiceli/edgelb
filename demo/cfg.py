import json

def route(matcher, response):
    return {"matcher": matcher, "response": response}

def header_matcher(header_name):
    return {"header": header_name}

def path_matcher(path_regex):
    return {"path": path_regex}

def http_redirect(code, location):
    return {"http_code": {"code": code, "location": location}}

x = {}

x["name"] = "test"
x["routes"] = []

x["routes"].append(route(header_matcher("X-mooch"), http_redirect(302, "https://omniflare.mooch.workers.dev/list")))
x["routes"].append(route(path_matcher("^/list$"), http_redirect(302, "https://omniflare.mooch.workers.dev/list")))

print(json.dumps(x))
