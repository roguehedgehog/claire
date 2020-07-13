# data "archive_file" "lambda_zip" {
#   type        = "zip"
#   output_path = "/tmp/lambdas.zip"
#   source_dir  = "../../bin/ir"
# }

# resource "aws_lambda_function" "lambda_zip" {
#   filename         = data.archive_file.lambda_zip.output_path
#   source_code_hash = data.archive_file.lambda_zip.output_base64sha256
# }
