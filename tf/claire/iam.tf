data "aws_iam_policy_document" "create_investigation" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:DescribeInstances",
      "ec2:CreateTags",
    ]
    resources = ["*"]
  }

  statement {
    effect = "Allow"
    actions = [
      "s3:PutObject"
    ]
    resources = [
      "${aws_s3_bucket.investigation_bucket.arn}/*"
    ]
  }
}

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

resource "aws_iam_role" "create_investigation" {
  name               = "claire_create_investigation"
  assume_role_policy = data.aws_iam_policy_document.lambda_execution_policy.json
}

resource "aws_iam_role_policy" "create_investigation" {
  name   = "create_investigation_role_policy"
  role   = aws_iam_role.create_investigation.id
  policy = data.aws_iam_policy_document.create_investigation.json
}
