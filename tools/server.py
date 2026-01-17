from flask import Flask, make_response

app = Flask(__name__)

@app.route("/set-cookie")
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

if __name__ == "__main__":
    app.run(debug=True)
