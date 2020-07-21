data "aws_iam_policy_document" "lambda_execution_policy" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "role" {
  name               = var.role_name
  assume_role_policy = data.aws_iam_policy_document.lambda_execution_policy.json
}

resource "aws_iam_role_policy" "policy" {
  name   = var.policy_name
  role   = aws_iam_role.role.id
  policy = var.policy_document
}

resource "aws_iam_role_policy_attachment" "allow_logging" {
  role       = aws_iam_role.role.id
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}
