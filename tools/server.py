from flask import Flask
from blueprints import cookies_bp

app = Flask(__name__)

app.register_blueprint(cookies_bp, url_prefix='/cookies')

if __name__ == "__main__":
    app.run(debug=True)
