resource "aws_s3_bucket" "investigation_bucket" {
  bucket = "${var.prefix}-investigations"
  server_side_encryption_configuration {
    rule {
      apply_server_side_encryption_by_default {
        sse_algorithm = "AES256"
      }
    }
  }
}
