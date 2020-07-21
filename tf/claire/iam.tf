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

module "create_investigation_role" {
  source = "./modules/iam_lambda"

  role_name       = "create_investigation_role"
  policy_name     = "create_investigation_policy"
  policy_document = data.aws_iam_policy_document.create_investigation.json
}

data "aws_iam_policy_document" "snapshot_disks" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:CreateSnapshot",
      "ec2:CreateTags",
      "ec2:DescribeInstances",
      "ec2:DescribeVolumes",
      "ec2:DescribeSnapshots",
    ]
    resources = ["*"]
  }
}

module "snapshot_disks_role" {
  source = "./modules/iam_lambda"

  role_name       = "snapshot_disks_role"
  policy_name     = "snapshot_disks_policy"
  policy_document = data.aws_iam_policy_document.snapshot_disks.json
}

data "aws_iam_policy_document" "isolate_instance" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:DescribeInstances",
      "ec2:ModifyInstanceAttribute",
    ]
    resources = ["*"]
  }
}

module "isolate_instance_role" {
  source = "./modules/iam_lambda"

  role_name       = "isolate_instance_role"
  policy_name     = "isolate_instance_policy"
  policy_document = data.aws_iam_policy_document.isolate_instance.json
}

resource "aws_iam_role" "claire_state_machine_role" {
  name               = "claire_state_machine_role"
  assume_role_policy = data.aws_iam_policy_document.claire_state_machine_policy.json
}

resource "aws_iam_role_policy" "claire_execute_lambda_policy" {
  name   = "claire_execute_lambda_policy"
  role   = aws_iam_role.claire_state_machine_role.id
  policy = data.aws_iam_policy_document.claire_state_machine_execute_lambda.json
}

data "aws_iam_policy_document" "claire_state_machine_execute_lambda" {
  statement {
    effect = "Allow"
    actions = [
      "lambda:InvokeFunction",
      "states:StartExecution",
    ]
    resources = ["*"]
  }
}

data "aws_iam_policy_document" "claire_state_machine_policy" {
  statement {
    actions = [
      "sts:AssumeRole",
    ]

    principals {
      type = "Service"
      identifiers = [
        "states.${var.aws_region}.amazonaws.com",
        "events.amazonaws.com",
      ]
    }
  }
}



