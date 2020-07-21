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



