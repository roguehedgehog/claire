resource "aws_s3_bucket" "attacker_bucket" {
  bucket = substr("${var.prefix}-lab-attacker-bucket-${random_id.attacker_bucket.hex}", 0, 63)
  acl    = "public-read-write"

  lifecycle_rule {
    enabled = true
    expiration {
      days = 1
    }
  }
}

resource "random_id" "attacker_bucket" {
  byte_length = 8
}

resource "aws_s3_bucket" "target_bucket" {
  bucket = "${var.prefix}-lab-target-bucket"
  acl    = "private"
}

resource "aws_s3_bucket_object" "docs" {
  for_each = fileset("./docs", "*")

  bucket = aws_s3_bucket.target_bucket.bucket
  key    = each.value
  source = "./docs/${each.value}"
  etag   = filemd5("./docs/${each.value}")
}
