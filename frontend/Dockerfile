FROM python:3.11

ADD . /app

WORKDIR /app

RUN pip3 install -r requirements.txt

ENTRYPOINT [ "python", "app.py" ]
