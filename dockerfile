FROM rust:latest

COPY ./ ./

EXPOSE 3000
CMD ["cargo" , "run", "--release"]