data "archive_file" "lambda_zip" {
  type        = "zip"
  output_path = "./dist/lambdas.zip"
  source_dir  = "../../bin/ir"
}

resource "aws_lambda_function" "create_investigation" {
  function_name = "claire_create_investigation"
  handler       = "create_investigation.lambda_handler"

  environment {
    variables = {
      INVESTIGATION_BUCKET = aws_s3_bucket.investigation_bucket.bucket
    }
  }

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
  role             = aws_iam_role.create_investigation.arn
}