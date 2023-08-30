FROM python:3.10
copy . /app
WORKDIR /app
RUN pip install --no-cache-dir --upgrade pip && pip install flask requests
CMD ["python3","/app/main.py"]
