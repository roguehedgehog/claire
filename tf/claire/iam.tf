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


data "aws_iam_policy_document" "create_instance" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:DescribeInstances",
      "ec2:DescribeInstanceTypes",
      "ec2:DescribeVolumes",
      "ec2:RunInstances",
      "ec2:AssociateIamInstanceProfile",
      "ec2:CreateTags",
      "iam:PassRole",
    ]
    resources = ["*"]
  }
}

module "create_instance_role" {
  source = "./modules/iam_lambda"

  role_name       = "create_instance_role"
  policy_name     = "create_instance_policy"
  policy_document = data.aws_iam_policy_document.create_instance.json
}

data "aws_iam_policy_document" "poll_instance" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:DescribeInstances",
    ]
    resources = ["*"]
  }
}

module "poll_instance_role" {
  source = "./modules/iam_lambda"

  role_name       = "poll_instance_role"
  policy_name     = "poll_instance_policy"
  policy_document = data.aws_iam_policy_document.poll_instance.json
}

data "aws_iam_policy_document" "terminate_instance" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:DescribeInstances",
      "ec2:TerminateInstances",
    ]
    resources = ["*"]
  }
}

module "terminate_instance_role" {
  source = "./modules/iam_lambda"

  role_name       = "terminate_instance_role"
  policy_name     = "terminate_instance_policy"
  policy_document = data.aws_iam_policy_document.terminate_instance.json
}

data "aws_iam_policy_document" "run_ssm_command" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:DescribeInstances",
      "ssm:SendCommand",
    ]
    resources = ["*"]
  }
}

module "run_ssm_command_role" {
  source = "./modules/iam_lambda"

  role_name       = "run_ssm_command_role"
  policy_name     = "run_ssm_command_policy"
  policy_document = data.aws_iam_policy_document.run_ssm_command.json
}

data "aws_iam_policy_document" "query_ssm_command" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:DescribeInstances",
      "ssm:getCommandInvocation",
    ]
    resources = ["*"]
  }
}

module "query_ssm_command_role" {
  source = "./modules/iam_lambda"

  role_name       = "query_ssm_command_role"
  policy_name     = "query_ssm_command_policy"
  policy_document = data.aws_iam_policy_document.query_ssm_command.json
}

data "aws_iam_policy_document" "manage_volume" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:DescribeInstances",
      "ec2:DescribeVolumes",
      "ec2:AttachVolume",
      "ec2:DetachVolume",
      "ec2:DeleteVolume",
    ]
    resources = ["*"]
  }
}

module "manage_volume_role" {
  source = "./modules/iam_lambda"

  role_name       = "manage_volume_role"
  policy_name     = "manage_volume_policy"
  policy_document = data.aws_iam_policy_document.manage_volume.json
}

data "aws_iam_policy_document" "query_volume" {
  statement {
    effect = "Allow"
    actions = [
      "ec2:DescribeInstances",
      "ec2:DescribeVolumes",
    ]
    resources = ["*"]
  }
}

module "query_volume_role" {
  source = "./modules/iam_lambda"

  role_name       = "query_volume_role"
  policy_name     = "query_volume_policy"
  policy_document = data.aws_iam_policy_document.query_volume.json
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
        "states.eu-west-1.amazonaws.com",
        "events.amazonaws.com",
      ]
    }
  }
}

data "aws_iam_policy" "CloudWatchEventsFullAccess" {
  arn = "arn:aws:iam::aws:policy/CloudWatchEventsFullAccess"
}

resource "aws_iam_role_policy_attachment" "claire_state_machine_role_cloud_watch" {
  role       = aws_iam_role.claire_state_machine_role.id
  policy_arn = data.aws_iam_policy.CloudWatchEventsFullAccess.arn
}

resource "aws_iam_instance_profile" "claire_ec2_evidence_extractor_profile" {
  name = "claire_ec2_evidence_extractor_profile"
  role = aws_iam_role.claire_ec2_evidence_extractor_role.name
}

resource "aws_iam_role" "claire_ec2_evidence_extractor_role" {
  name               = "claire_ec2_evidence_extractor_role"
  assume_role_policy = data.aws_iam_policy_document.claire_ec2_policy.json
}

data "aws_iam_policy_document" "claire_ec2_policy" {
  statement {
    actions = [
      "sts:AssumeRole",
    ]

    principals {
      type = "Service"
      identifiers = [
        "ec2.amazonaws.com",
      ]
    }
  }
}

resource "aws_iam_role_policy" "claire_ec2_evidence_extractor_s3_access" {
  name   = "claire_ec2_evidence_extractor_ssm_access"
  role   = aws_iam_role.claire_ec2_evidence_extractor_role.id
  policy = data.aws_iam_policy_document.create_investigation.json
}

data "aws_iam_policy" "AmazonEc2RoleForSSM" {
  arn = "arn:aws:iam::aws:policy/service-role/AmazonEC2RoleforSSM"
}

resource "aws_iam_role_policy_attachment" "evidence_extractor_ssm_access" {
  role       = aws_iam_role.claire_ec2_evidence_extractor_role.id
  policy_arn = data.aws_iam_policy.AmazonEc2RoleForSSM.arn
}
