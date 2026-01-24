from flask import Blueprint, make_response
cookies_bp = Blueprint('cookies', __name__)


@cookies_bp.get("/set-cookie")
def set_cookie():
    resp = make_response("Cookie has been set")
    resp.set_cookie(
        key="example_cookie",
        value="hello_worldagain",
        max_age=60 * 60,
        httponly=True,
        secure=False,
        samesite="Lax"
    )
    return resp
