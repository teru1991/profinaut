# docker/vault/scripts/Dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY ./scripts/export_secret_expiry.py ./
RUN pip install flask requests
ENV FLASK_ENV=production
CMD ["python", "export_secret_expiry.py"]